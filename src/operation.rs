pub enum OperationType {
  Queue,
  Dequeue,
  AddNode,
  Commit,
  None
}

pub struct Operation {
  operation_type: OperationType,
  content: Option<String>
}