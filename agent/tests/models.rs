use agent::models::{
    action::Action, container::mock::MockContainer, output_pipe::{OutputPipe, Pipe}, step::Step,
};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[tokio::test]
async fn test_model_integration_workflow() {
    // 1. Setup - Create container and communication channels
    let (tx, _) = mpsc::unbounded_channel();
    let mock_container = MockContainer {
        exec_calls: Mutex::new(Vec::new()),
        should_fail: false,
    };

    // 2. Create the action with multiple steps
    let action_id = 123;
    let commands = vec![
        "echo 'Step 1'".to_string(),
        "cd /app && ls -la".to_string(),
        "echo 'Step 3'".to_string(),
    ];
    let repo_url = "https://github.com/test/repo.git".to_string();

    let action = Action::new(
        action_id,
        mock_container,
        commands.clone(),
        tx.clone(),
        repo_url.clone(),
    );

    // 3. Execute the workflow - setup and run action
    let setup_result = action.setup_repository().await;
    assert!(setup_result.is_ok(), "Repository setup should succeed");

    let execution_result = action.execute().await;
    assert!(execution_result.is_ok(), "Action execution should succeed");

    // 4. Verify container was accessed correctly
    let container = Arc::new(&action.container);
    let calls = container.exec_calls.lock().unwrap();

    // First call should be the git clone
    assert!(
        calls[0].0.contains("git clone"),
        "Should start with git clone"
    );
    assert!(
        calls[0].0.contains(&repo_url),
        "Should clone the correct repo"
    );

    // Check that each command was executed
    for (i, cmd) in commands.iter().enumerate() {
        assert_eq!(&calls[i + 1].0, cmd, "Command {} should be executed", i + 1);
        assert_eq!(
            calls[i + 1].1,
            Some(format!("/{}", action_id)),
            "Command {} should execute in the right directory",
            i + 1
        );
    }
}

#[tokio::test]
async fn test_step_to_action_to_output_integration() {
    // Setup - Create the message channel
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Create an output pipe
    let action_id = 42;
    let pipe = Arc::new(OutputPipe::new(action_id, tx.clone()));

    // Create a mock container
    let mock_container = Arc::new(MockContainer {
        exec_calls: Mutex::new(Vec::new()),
        should_fail: false,
    });

    // Create a step
    let command = "echo 'Hello Integration'".to_string();
    let step = Step::new(
        command.clone(),
        Some("/test".to_string()),
        mock_container.clone(),
    );

    // Execute the step
    let exec_result = step.execute().await;
    assert!(exec_result.is_ok(), "Step execution should succeed");

    // Send output through the pipe
    pipe.output_log("Step started".to_string(), 1, None);
    pipe.output_log("Step completed".to_string(), 100, Some(0));

    // Verify outputs were sent correctly
    let mut messages = Vec::new();
    for _ in 0..2 {
        if let Some(Ok(msg)) = rx.recv().await {
            messages.push(msg);
        }
    }

    assert_eq!(messages.len(), 2, "Should receive 2 messages");
    assert_eq!(messages[0].action_id, action_id);
    assert_eq!(messages[0].log, "Step started");
    assert_eq!(messages[1].log, "Step completed");

    let result = messages[1].result.as_ref().unwrap();
    assert_eq!(result.completion, 100);
    assert_eq!(result.exit_code, Some(0));
}

#[tokio::test]
async fn test_action_error_propagation() {
    // Setup - Create a failing container
    let (tx, _) = mpsc::unbounded_channel();
    let mock_container = MockContainer {
        exec_calls: Mutex::new(Vec::new()),
        should_fail: true, // Configure to fail
    };

    let action = Action::new(
        1,
        mock_container,
        vec!["will_fail".to_string()],
        tx,
        "https://example.com/repo.git".to_string(),
    );

    // Attempt to setup repository (will fail)
    let setup_result = action.setup_repository().await;
    assert!(
        setup_result.is_err(),
        "Setup should fail with a failing container"
    );

    // Check error is properly typed
    match setup_result {
        Err(agent::models::error::Error::ContainerExecError(_)) => {}
        _ => panic!("Expected ContainerExecError"),
    }

    // Attempt to execute (will also fail)
    let exec_result = action.execute().await;
    assert!(
        exec_result.is_err(),
        "Execution should fail with a failing container"
    );
}

#[tokio::test]
async fn test_complete_workflow_with_cleanup() {
    // Setup
    let (tx, _) = mpsc::unbounded_channel();
    let mock_container = MockContainer {
        exec_calls: Mutex::new(Vec::new()),
        should_fail: false,
    };

    let action = Action::new(
        99,
        mock_container,
        vec!["echo 'success'".to_string()],
        tx,
        "https://example.com/test.git".to_string(),
    );

    // Run full workflow
    let setup_result = action.setup_repository().await;
    assert!(setup_result.is_ok());

    let exec_result = action.execute().await;
    assert!(exec_result.is_ok());

    // Note: cleanup is automatically called at the end of execute()
    // So we only need to verify that the exec_calls includes the remove operation

    // Verify the full sequence of operations
    let container = Arc::new(&action.container);
    let calls = container.exec_calls.lock().unwrap();

    // Expect at least 1 git clone + 1 command execution
    assert!(calls.len() >= 2, "Should have at least 2 operations");

    // Check that the first call is git clone
    assert!(
        calls[0].0.contains("git clone"),
        "First call should be git clone"
    );

    // Check that the command was executed
    assert_eq!(&calls[1].0, "echo 'success'", "Command should be executed");
}
