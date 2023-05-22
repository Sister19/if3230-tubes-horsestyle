use crate::prelude::*;

/*
- /execute
    - struct Request {Operation: enum (Queue, Dequeue), Content: String}
    - kalo leader:
        - kirim /operation ke semua follower
            - cek apakah ada response mengandung INCONSISTENT_LOG
                - kalo ada diloop sampe accept
                    - last operation
                    - kalau ada, kirim /operation dengan Operations = Log[last_operation-1: last_operation], last_operation = last_operation - 1 PreviousLogEntry = Log[last_operation - 1]
        - kalau 50% + 1 ack, commit
            - kirim /operation commit
    - else:
        - tolak 
*/
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ExecuteRequest{
  pub operation_type: OperationType,
  pub content: Option<String>,
}

// Prekondisi : operation_to_execute nya hanya Queue dan Dequeue
pub async fn execute(context: web::Data<Arc<Mutex<NodeInfo>>>, operation_to_execute: web::Json<Operation>) -> impl Responder {
  let mut ctx: MutexGuard<NodeInfo> = context.lock().unwrap();

  let operation = &Operation {
    operation_type: operation_to_execute.operation_type.clone(),
    content: Some(String::from(operation_to_execute.content.clone())),
    is_committed: None
  };
  if (ctx.node_type == NodeType::Leader){
    // kirim /operation ke semua follower
    let operation_request = &OperationRequest{
      operations: vec![operation.clone()],
      sender: ctx.address.clone(),
      previous_log_entry: None,
      term: ctx.term.clone(),
    };

    let responses = post_many(ctx.peers.clone(), OPERATION_ROUTE, &serde_json::to_string(&operation_request).unwrap()).await;
        
        let mut count = 0;
        for mut response in responses {
          match response {
            Ok(sk) => {
              let operation_response = sk.json::<OperationResponse>().await.unwrap();
              if operation_response.accepted {
                count += 1;
              }else{
                if (operation_response.flag == OperationError::InconsistentLog){
                  // ... Masalah blocking dan cuma ngirim operation (termnya kenapa ngga)
                }
              }
            },
            Err(e) => {
              print!("{:?}", e);
            }
          }
        }
        if (count >= ctx.peers.len() / 2 + 1) { // handle commit yang gagal di term sebelumnya(?) - Pastiin ke Roby
          let commit_operation = &Operation {
            operation_type: OperationType::Commit,
            content: None,
            is_committed: true
          };
          let commit_request= &OperationRequest{
            operations: vec![commit_operation.clone()],
            sender: ctx.address.clone(),
            previous_log_entry: None,
            term: ctx.term.clone(),
          };
          post_many(ctx.peers.clone(), OPERATION_ROUTE, &serde_json::to_string(&commit_request).unwrap())
        }
    // request log ke tiap follower

    // Cek Inconsistent Log
      // Jika Ya
        // Fungsi Consistency Check Seperti di Slide
      // Jika tidak
        // Lanjut
    
    // Commit Operation yang bersesuaian untuk tiap follower
  }
}
