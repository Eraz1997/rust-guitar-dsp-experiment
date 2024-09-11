use crate::managers::dsp::error::TransformProcessorError;
use crate::managers::dsp::models::{ProcessorParameters, ProcessorType};
use crate::managers::dsp::DSPManager;
use crate::managers::file_system::error::Error as FileSystemError;
use crate::managers::file_system::FileSystemManager;
use crate::processors::frontline::create_processor_from_type;
use crate::processors::frontline::models::Parameter;
use crate::routes::api::v1::processors::models::requests::{
    CreateProcessorRequest, EditParameterRequest, MoveProcessorRequest,
    SetProcessorBypassedRequest, SwapProcessorRequest,
};
use crate::routes::api::v1::processors::models::responses::{
    CreateProcessorResponse, GetStringParameterValuesResponse, SwapProcessorResponse,
};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use axum::{Extension, Json, Router};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

mod models;

pub fn create_router() -> Router {
    Router::new()
        .route("/", post(create_processor))
        .route("/:processor_index", put(swap_processor))
        .route("/:processor_index", delete(delete_processor))
        .route("/:processor_index/bypassed", put(set_processor_bypassed))
        .route("/:processor_index/move", put(move_processor))
        .route(
            "/:processor_index/parameters/:parameter",
            put(edit_parameter),
        )
        .route(
            "/:processor_type/parameters",
            get(get_string_parameter_values),
        )
}

async fn create_processor(
    dsp_manager: Extension<Arc<Mutex<DSPManager>>>,
    Json(payload): Json<CreateProcessorRequest>,
) -> Json<CreateProcessorResponse> {
    let mut dsp_manager = dsp_manager.lock().unwrap();
    let (numeric_parameters, string_parameters) =
        add_new_processor(&mut dsp_manager, payload.index, payload.processor_type);

    Json(CreateProcessorResponse {
        parameters: ProcessorParameters {
            numeric: numeric_parameters,
            string: string_parameters,
        },
    })
}

async fn swap_processor(
    dsp_manager: Extension<Arc<Mutex<DSPManager>>>,
    Path(processor_index): Path<usize>,
    Json(payload): Json<SwapProcessorRequest>,
) -> Json<SwapProcessorResponse> {
    let mut dsp_manager = dsp_manager.lock().unwrap();
    dsp_manager.extract_processor(processor_index);

    let (numeric_parameters, string_parameters) =
        add_new_processor(&mut dsp_manager, processor_index, payload.processor_type);

    Json(SwapProcessorResponse {
        parameters: ProcessorParameters {
            numeric: numeric_parameters,
            string: string_parameters,
        },
    })
}

async fn delete_processor(
    dsp_manager: Extension<Arc<Mutex<DSPManager>>>,
    Path(processor_index): Path<usize>,
) {
    let mut dsp_manager = dsp_manager.lock().unwrap();
    dsp_manager.extract_processor(processor_index);
}

async fn move_processor(
    dsp_manager: Extension<Arc<Mutex<DSPManager>>>,
    Path(processor_index): Path<usize>,
    Json(payload): Json<MoveProcessorRequest>,
) {
    let mut dsp_manager = dsp_manager.lock().unwrap();
    let processor = dsp_manager.extract_processor(processor_index);

    let destination_index = if payload.destination_index <= processor_index {
        payload.destination_index
    } else {
        payload.destination_index - 1
    };
    dsp_manager.add_processor(destination_index, processor);
}

async fn set_processor_bypassed(
    dsp_manager: Extension<Arc<Mutex<DSPManager>>>,
    Path(processor_index): Path<usize>,
    Json(payload): Json<SetProcessorBypassedRequest>,
) -> Result<(), StatusCode> {
    let dsp_manager = dsp_manager.lock().unwrap();
    match dsp_manager.transform_processor_settings(processor_index, |processor_settings| {
        processor_settings.bypassed = payload.bypassed
    }) {
        Ok(_) => Ok(()),
        Err(TransformProcessorError::NotFound) => Err(StatusCode::NOT_FOUND),
    }
}

async fn edit_parameter(
    dsp_manager: Extension<Arc<Mutex<DSPManager>>>,
    Path((processor_index, parameter)): Path<(usize, Parameter)>,
    Json(payload): Json<EditParameterRequest>,
) -> Result<(), StatusCode> {
    let dsp_manager = dsp_manager.lock().unwrap();
    match dsp_manager.transform_processor(processor_index, |processor| {
        processor.set_parameter(parameter, payload.value)
    }) {
        Ok(_) => Ok(()),
        Err(TransformProcessorError::NotFound) => Err(StatusCode::NOT_FOUND),
    }
}

async fn get_string_parameter_values(
    file_system_manager: Extension<FileSystemManager>,
    Path(processor_type): Path<ProcessorType>,
) -> Result<Json<GetStringParameterValuesResponse>, StatusCode> {
    if let Some(directory_name) = match processor_type {
        ProcessorType::IR => Some("impulseResponses".to_string()),
        ProcessorType::Clone => Some("clones".to_string()),
        _ => None,
    } {
        let available_values = file_system_manager
            .get_directory_names_in_directory(directory_name)?
            .into_iter()
            .flat_map(|category| {
                file_system_manager
                    .get_file_names_in_directory(category)
                    .unwrap_or_default()
            })
            .collect();
        let mut values = HashMap::new();
        values.insert(Parameter::FilePath, available_values);
        Ok(Json(GetStringParameterValuesResponse { values }))
    } else {
        Ok(Json(GetStringParameterValuesResponse {
            values: HashMap::new(),
        }))
    }
}

fn add_new_processor(
    dsp_manager: &mut MutexGuard<DSPManager>,
    index: usize,
    processor_type: ProcessorType,
) -> (HashMap<Parameter, f32>, HashMap<Parameter, String>) {
    let processor = create_processor_from_type(
        &processor_type,
        &dsp_manager.sample_rate,
        &dsp_manager.buffer_size,
    );
    let numeric_parameters = processor.get_numeric_parameters();
    let string_parameters = processor.get_string_parameters();
    dsp_manager.add_processor(index, processor);

    (numeric_parameters, string_parameters)
}

impl From<FileSystemError> for StatusCode {
    fn from(value: FileSystemError) -> Self {
        match value {
            FileSystemError::Conversion => StatusCode::INTERNAL_SERVER_ERROR,
            FileSystemError::Generic(_) => StatusCode::INTERNAL_SERVER_ERROR,
            FileSystemError::HomeDirectoryNotFound => StatusCode::NOT_FOUND,
            FileSystemError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}
