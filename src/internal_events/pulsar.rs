use metrics::counter;
use metrics::{register_counter, register_histogram, Counter, Histogram};
use vector_core::internal_event::InternalEvent;

use crate::emit;
use vector_common::{
    internal_event::{error_stage, error_type, ComponentEventsDropped, UNINTENTIONAL},
    registered_event,
};

#[derive(Debug)]
pub struct PulsarSendingError {
    pub count: usize,
    pub error: vector_common::Error,
}

impl InternalEvent for PulsarSendingError {
    fn emit(self) {
        let reason = "A Pulsar sink generated an error.";
        error!(
            message = reason,
            error = %self.error,
            error_type = error_type::REQUEST_FAILED,
            stage = error_stage::SENDING,
            internal_log_rate_limit = true,
        );
        counter!(
            "component_errors_total", 1,
            "error_type" => error_type::REQUEST_FAILED,
            "stage" => error_stage::SENDING,
        );
        emit!(ComponentEventsDropped::<UNINTENTIONAL> {
            count: self.count,
            reason,
        });
    }
}

pub struct PulsarPropertyExtractionError<F: std::fmt::Display> {
    pub property_field: F,
}

impl<F: std::fmt::Display> InternalEvent for PulsarPropertyExtractionError<F> {
    fn emit(self) {
        error!(
            message = "Failed to extract properties. Value should be a map of String -> Bytes.",
            error_code = "extracting_property",
            error_type = error_type::PARSER_FAILED,
            stage = error_stage::PROCESSING,
            property_field = %self.property_field,
            internal_log_rate_limit = true,
        );
        counter!(
            "component_errors_total", 1,
            "error_code" => "extracting_property",
            "error_type" => error_type::PARSER_FAILED,
            "stage" => error_stage::PROCESSING,
        );
    }
}

pub enum PulsarErrorEventType {
    ReadError,
    AckError,
    NAckError,
}

pub struct PulsarErrorEventData {
    pub msg: String,
    pub error_type:PulsarErrorEventType,
}

registered_event!(
    PulsarErrorEvent => {
        ack_errors_count: Histogram = register_histogram!(
            "component_errors_count",
            "error_code" => "acknowledge_message",
            "error_type" => error_type::ACKNOWLEDGMENT_FAILED,
            "stage" => error_stage::RECEIVING,
        ),
        ack_errors: Counter = register_counter!(
            "component_errors_total",
            "error_code" => "acknowledge_message",
            "error_type" => error_type::ACKNOWLEDGMENT_FAILED,
            "stage" => error_stage::RECEIVING,
        ),

        nack_errors_count: Histogram = register_histogram!(
            "component_errors_count",
            "error_code" => "negative_acknowledge_message",
            "error_type" => error_type::ACKNOWLEDGMENT_FAILED,
            "stage" => error_stage::RECEIVING,
        ),
        nack_errors: Counter = register_counter!(
            "component_errors_total",
            "error_code" => "negative_acknowledge_message",
            "error_type" => error_type::ACKNOWLEDGMENT_FAILED,
            "stage" => error_stage::RECEIVING,
        ),

        read_errors_count: Histogram = register_histogram!(
            "component_errors_count",
            "error_code" => "reading_message",
            "error_type" => error_type::READER_FAILED,
            "stage" => error_stage::RECEIVING,
        ),
        read_errors: Counter = register_counter!(
            "component_errors_total",
            "error_code" => "reading_message",
            "error_type" => error_type::READER_FAILED,
            "stage" => error_stage::RECEIVING,
        ),
    }

    fn emit(&self,error:PulsarErrorEventData) {
        match error.error_type{
            PulsarErrorEventType::ReadError=>{
                error!(
                    message = "Failed to read message.",
                    error = error.msg,
                    error_code = "reading_message",
                    error_type = error_type::READER_FAILED,
                    stage = error_stage::RECEIVING,
                    internal_log_rate_limit = true,
                );

                self.read_errors_count.record(1_f64);
                self.read_errors.increment(1_u64);
            }
            PulsarErrorEventType::AckError=>{
                error!(
                    message = "Failed to acknowledge message.",
                    error = error.msg,
                    error_code = "acknowledge_message",
                    error_type = error_type::ACKNOWLEDGMENT_FAILED,
                    stage = error_stage::RECEIVING,
                    internal_log_rate_limit = true,
                );

                self.ack_errors_count.record(1_f64);
                self.ack_errors.increment(1_u64);
            }
            PulsarErrorEventType::NAckError=>{
                error!(
                    message = "Failed to negatively acknowledge message.",
                    error = error.msg,
                    error_code = "negative_acknowledge_message",
                    error_type = error_type::ACKNOWLEDGMENT_FAILED,
                    stage = error_stage::RECEIVING,
                    internal_log_rate_limit = true,
                );

                self.nack_errors_count.record(1_f64);
                self.nack_errors.increment(1_u64);
            }
        }
    }
);
