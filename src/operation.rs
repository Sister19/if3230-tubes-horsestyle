#[derive(Clone)]
pub enum OperationType {
  Queue,
  Dequeue,
  AddNode,
  Commit,
  None
}

#[derive(Clone)]
pub struct Operation {
  operation_type: OperationType,
  content: Option<String>
}