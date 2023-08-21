// Copyright (c) 2023 -  Restate Software, Inc., Restate GmbH.
// All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

//! A Restate journal is represented by Restate entries, each of them recording a specific action taken by the user code.

use super::*;

use crate::errors::UserErrorCode;
use crate::identifiers::EntryIndex;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    // IO
    PollInputStream(PollInputStreamEntry),
    OutputStream(OutputStreamEntry),

    // State access
    GetState(GetStateEntry),
    SetState(SetStateEntry),
    ClearState(ClearStateEntry),

    // Syscalls
    Sleep(SleepEntry),
    Invoke(InvokeEntry),
    BackgroundInvoke(BackgroundInvokeEntry),
    Awakeable(AwakeableEntry),
    CompleteAwakeable(CompleteAwakeableEntry),
    Custom(Bytes),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    pub entry_index: EntryIndex,
    pub result: CompletionResult,
}

impl Completion {
    pub fn new(entry_index: EntryIndex, result: CompletionResult) -> Self {
        Self {
            entry_index,
            result,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionResult {
    Ack,
    Empty,
    Success(Bytes),
    Failure(UserErrorCode, ByteString),
}

impl From<ResponseResult> for CompletionResult {
    fn from(value: ResponseResult) -> Self {
        match value {
            ResponseResult::Success(bytes) => CompletionResult::Success(bytes),
            ResponseResult::Failure(error_code, error_msg) => {
                CompletionResult::Failure(error_code, error_msg)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
    PollInputStream,
    OutputStream,
    GetState,
    SetState,
    ClearState,
    Sleep,
    Invoke,
    BackgroundInvoke,
    Awakeable,
    CompleteAwakeable,
    Custom,
}

impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryResult {
    Success(Bytes),
    Failure(UserErrorCode, ByteString),
}

impl From<EntryResult> for ResponseResult {
    fn from(value: EntryResult) -> Self {
        match value {
            EntryResult::Success(bytes) => ResponseResult::Success(bytes),
            EntryResult::Failure(code, error_msg) => ResponseResult::Failure(code, error_msg),
        }
    }
}

pub trait CompletableEntry: private::Sealed {
    /// Returns true if the entry is completed.
    fn is_completed(&self) -> bool;
}

mod private {
    use super::*;

    pub trait Sealed {}
    impl Sealed for GetStateEntry {}
    impl Sealed for SleepEntry {}
    impl Sealed for InvokeEntry {}
    impl Sealed for AwakeableEntry {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PollInputStreamEntry {
    pub result: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputStreamEntry {
    pub result: EntryResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GetStateValue {
    Empty,
    Value(Bytes),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetStateEntry {
    pub key: Bytes,
    pub value: Option<GetStateValue>,
}

impl CompletableEntry for GetStateEntry {
    fn is_completed(&self) -> bool {
        self.value.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetStateEntry {
    pub key: Bytes,
    pub value: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClearStateEntry {
    pub key: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SleepEntry {
    pub wake_up_time: u64,
    pub fired: bool,
}

impl CompletableEntry for SleepEntry {
    fn is_completed(&self) -> bool {
        self.fired
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvokeRequest {
    pub service_name: ByteString,
    pub method_name: ByteString,
    pub parameter: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvokeEntry {
    pub request: InvokeRequest,
    pub result: Option<EntryResult>,
}

impl CompletableEntry for InvokeEntry {
    fn is_completed(&self) -> bool {
        self.result.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundInvokeEntry {
    pub request: InvokeRequest,
    pub invoke_time: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwakeableEntry {
    pub result: Option<EntryResult>,
}

impl CompletableEntry for AwakeableEntry {
    fn is_completed(&self) -> bool {
        self.result.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteAwakeableEntry {
    pub id: ByteString,
    pub result: EntryResult,
}
