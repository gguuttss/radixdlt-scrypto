use radix_engine::{
    errors::{ApplicationError, RuntimeError},
    transaction::TransactionReceipt,
    types::*,
};
use radix_engine_interface::types::Level;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

fn call<S: AsRef<str>>(function_name: &str, message: S) -> TransactionReceipt {
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/logger");

    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "Logger",
            function_name,
            manifest_args!(message.as_ref().to_owned()),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    receipt
}

#[test]
fn test_log_message() {
    // Arrange
    let function_name = "log_message";
    let message = "Hello World";

    // Act
    let receipt = call(function_name, message);

    // Assert
    {
        receipt.expect_commit_success();

        let logs = receipt.expect_commit(true).application_logs.clone();
        let expected_logs = vec![(Level::Info, message.to_owned())];

        assert_eq!(expected_logs, logs)
    }
}

#[test]
fn test_rust_panic() {
    // Arrange
    let function_name = "rust_panic";
    let message = "Hey Hey World";

    // Act
    let receipt = call(function_name, message);

    // Assert
    {
        let logs = receipt.expect_commit(false).application_logs.clone();
        let expected_logs = vec![(
            Level::Error,
            "Panicked at 'I'm panicking!', logger/src/lib.rs:15:13".to_owned(),
        )];

        assert_eq!(expected_logs, logs)
    }
}

#[test]
fn test_scrypto_panic() {
    // Arrange
    let function_name = "scrypto_panic";
    let message = "Hi";

    // Act
    let receipt = call(function_name, message);

    // Assert
    {
        let logs = receipt.expect_commit(false).application_logs.clone();
        assert!(logs.is_empty());

        receipt.expect_specific_failure(|e| match e {
            RuntimeError::ApplicationError(ApplicationError::Panic(e)) if e.eq(message) => true,
            _ => false,
        })
    }
}

#[test]
fn test_assert_length_5() {
    // Arrange
    let function_name = "assert_length_5";
    let message = "Message not of length 5";

    // Act
    let receipt = call(function_name, message);

    // Assert
    {
        let logs = receipt.expect_commit(false).application_logs.clone();
        let expected_logs = vec![
            (
                Level::Error,
                "Panicked at 'assertion failed: `(left == right)`\n  left: `23`,\n right: `5`', logger/src/lib.rs:23:13".to_owned(),
            ),
        ];

        assert_eq!(expected_logs, logs)
    }
}
