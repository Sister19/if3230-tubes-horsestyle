use crate::prelude::*;

#[derive(Clone, PartialEq, Eq, Deserialize, Serialize, Debug)]
pub enum OperationType {
  Queue,
  Dequeue,
  AddNode,
  Commit,
  None
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Operation {
  pub operation_type: OperationType,
  pub content: Option<String>,
  pub is_committed: Option<bool> 
}

impl Operation {
  pub fn new(operation_type: OperationType, content: String) -> Self {
    Operation {
      operation_type: operation_type,
      content: Some(content),
      is_committed: Some(false)
    }
  }

  pub fn is_equal(&self, operation: Operation) -> bool {
    if (operation.operation_type == self.operation_type) && (operation.content == self.content) {
      return true;
    }
    return false;
  }
}