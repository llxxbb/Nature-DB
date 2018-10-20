use nature_common::NatureError;
use std::convert::TryFrom;

pub enum TaskType {
    Store = 1,
    Convert = 2,
    ParallelBatch = 11,
    QueueBatch = 12,
}

impl TryFrom<i16> for TaskType {
    type Error = NatureError;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(TaskType::Store),
            2 => Ok(TaskType::Convert),
            11 => Ok(TaskType::ParallelBatch),
            12 => Ok(TaskType::QueueBatch),
            _ => Err(NatureError::VerifyError(format!("undefined [{}] for `TaskType`", value)))
        }
    }
}
