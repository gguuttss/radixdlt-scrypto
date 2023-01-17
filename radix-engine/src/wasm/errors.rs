use radix_engine_interface::api::wasm::BufferId;
use wasmi::HostError;

use crate::fee::FeeReserveError;
use crate::model::InvokeError;
use crate::types::*;

/// Represents an error when validating a WASM file.
#[derive(Debug, PartialEq, Eq, Clone, Categorize, Encode, Decode)]
pub enum PrepareError {
    /// Failed to deserialize.
    /// See <https://webassembly.github.io/spec/core/syntax/index.html>
    DeserializationError,
    /// Failed to validate
    /// See <https://webassembly.github.io/spec/core/valid/index.html>
    ValidationError,
    /// Failed to serialize.
    SerializationError,
    /// The wasm module contains a start function.
    StartFunctionNotAllowed,
    /// The wasm module uses float points.
    FloatingPointNotAllowed,
    /// Invalid import section
    InvalidImport(InvalidImport),
    /// Invalid memory section
    InvalidMemory(InvalidMemory),
    /// Invalid table section
    InvalidTable(InvalidTable),
    /// Too many targets in the `br_table` instruction
    TooManyTargetsInBrTable,
    /// Too many functions
    TooManyFunctions,
    /// Too many globals
    TooManyGlobals,
    /// No export section
    NoExportSection,
    /// Missing export
    MissingExport { export_name: String },
    /// The wasm module does not have the `scrypto_alloc` export.
    NoScryptoAllocExport,
    /// The wasm module does not have the `scrypto_free` export.
    NoScryptoFreeExport,
    /// Failed to inject instruction metering
    RejectedByInstructionMetering,
    /// Failed to inject stack metering
    RejectedByStackMetering,
    /// Not instantiatable
    NotInstantiatable,
    /// Not compilable
    NotCompilable,
}

#[derive(Debug, PartialEq, Eq, Clone, Categorize, Encode, Decode)]
pub enum InvalidImport {
    /// The import is not allowed
    ImportNotAllowed,
}

#[derive(Debug, PartialEq, Eq, Clone, Categorize, Encode, Decode)]
pub enum InvalidMemory {
    /// The wasm module has no memory section.
    NoMemorySection,
    /// The memory section is empty.
    EmptyMemorySection,
    /// The memory section contains too many memory definitions.
    TooManyMemories,
    /// The initial memory size is too large.
    InitialMemorySizeLimitExceeded,
    /// The wasm module does not have the `memory` export.
    MemoryNotExported,
}

#[derive(Debug, PartialEq, Eq, Clone, Categorize, Encode, Decode)]
pub enum InvalidTable {
    /// More than one table defined, against WebAssembly MVP spec
    MoreThanOneTable,
    /// Initial table size too large
    InitialTableSizeLimitExceeded,
}

/// Represents an error when invoking an export of a Scrypto module.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum WasmShimError {
    /// Error when reading from wasm memory.
    MemoryAccessError,

    /// WASM attempted to call undefined function index.
    UnknownFunctionIndex(usize),

    /// WASM interpreter error, such as traps.
    InterpreterError(String),

    //=============
    // SHIM ERRORS
    //=============
    /// Not implemented, no-op wasm runtime
    NotImplemented,
    /// Buffer not found
    BufferNotFound(BufferId),
    /// Invalid scrypto receiver
    InvalidReceiver(DecodeError),
    /// Invalid invocation
    InvalidInvocation(DecodeError),
    /// Invalid RE node data
    InvalidNode(DecodeError),
    /// Invalid RE node ID
    InvalidNodeId(DecodeError),
    /// Invalid substate offset
    InvalidOffset(DecodeError),
}

impl fmt::Display for WasmShimError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl HostError for WasmShimError {}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for WasmShimError {}

impl fmt::Display for InvokeError<WasmShimError> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl HostError for InvokeError<WasmShimError> {}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for InvokeError<WasmShimError> {}
