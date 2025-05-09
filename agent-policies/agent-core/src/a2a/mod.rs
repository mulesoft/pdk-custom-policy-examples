use crate::a2a::schemas::{ TaskIdParams, TaskPushNotificationConfig, TaskQueryParams, TaskSendParams};
use crate::json_rpc::JsonRpcRequest;

pub mod schemas;

pub const SEND_TASK_FUNCTION_NAME: &'static str = "tasks/send";
pub const GET_TASK_FUNCTION_NAME: &'static str = "tasks/get";
pub const CANCEL_TASK_FUNCTION_NAME: &'static str = "tasks/cancel";
pub const SET_PUSH_NOTIFICATION_TASK_FUNCTION_NAME: &'static str = "tasks/pushNotification/set";
pub const GET_PUSH_NOTIFICATION_TASK_FUNCTION_NAME: &'static str = "tasks/pushNotification/get";
pub const SEND_SUBSCRIBE_TASK_FUNCTION_NAME: &'static str = "tasks/sendSubscribe";

pub fn is_valid_method(name: &str) -> bool {
    valid_methods().contains(&name)
}

pub fn valid_methods() -> Vec<&'static str> {
    vec![
        SEND_TASK_FUNCTION_NAME,
        GET_TASK_FUNCTION_NAME,
        CANCEL_TASK_FUNCTION_NAME,
        SET_PUSH_NOTIFICATION_TASK_FUNCTION_NAME,
        GET_PUSH_NOTIFICATION_TASK_FUNCTION_NAME,
        SEND_SUBSCRIBE_TASK_FUNCTION_NAME,
    ]
}

pub fn valid_request(request: JsonRpcRequest<'_>) -> Result<(), anyhow::Error> {
    match request.method {
        SEND_TASK_FUNCTION_NAME => match request.params {
            None => Err(anyhow::Error::msg(format!(
                "Missing `{}` params",
                SEND_TASK_FUNCTION_NAME
            ))),
            Some(p) => {
                serde_json::from_str::<TaskSendParams>(p.get())
                    .map_err(|e| anyhow::Error::from(e))?;
                Ok(())
            }
        },
        GET_TASK_FUNCTION_NAME => match request.params {
            None => Err(anyhow::Error::msg(format!(
                "Missing `{}` params",
                GET_TASK_FUNCTION_NAME
            ))),
            Some(p) => {
                serde_json::from_str::<TaskQueryParams>(p.get())
                    .map_err(|e| anyhow::Error::from(e))?;
                Ok(())
            }
        },
        CANCEL_TASK_FUNCTION_NAME => match request.params {
            None => Err(anyhow::Error::msg(format!(
                "Missing `{}` params",
                CANCEL_TASK_FUNCTION_NAME
            ))),
            Some(p) => {
                serde_json::from_str::<TaskIdParams>(p.get())
                    .map_err(|e| anyhow::Error::from(e))?;
                Ok(())
            }
        },
        SET_PUSH_NOTIFICATION_TASK_FUNCTION_NAME => match request.params {
            None => Err(anyhow::Error::msg(format!(
                "Missing `{}` params",
                SET_PUSH_NOTIFICATION_TASK_FUNCTION_NAME
            ))),
            Some(p) => {
                serde_json::from_str::<TaskPushNotificationConfig>(p.get())
                    .map_err(|e| anyhow::Error::from(e))?;
                Ok(())
            }
        },
        GET_PUSH_NOTIFICATION_TASK_FUNCTION_NAME => match request.params {
            None => Err(anyhow::Error::msg(format!(
                "Missing `{}` params",
                GET_PUSH_NOTIFICATION_TASK_FUNCTION_NAME
            ))),
            Some(p) => {
                serde_json::from_str::<TaskIdParams>(p.get())
                    .map_err(|e| anyhow::Error::from(e))?;
                Ok(())
            }
        },
        SEND_SUBSCRIBE_TASK_FUNCTION_NAME => match request.params {
            None => Err(anyhow::Error::msg(format!(
                "Missing `{}` params",
                SEND_SUBSCRIBE_TASK_FUNCTION_NAME
            ))),
            Some(p) => {
                serde_json::from_str::<TaskQueryParams>(p.get())
                    .map_err(|e| anyhow::Error::from(e))?;
                Ok(())
            }
        },
        _ => Ok(()),
    }
}
