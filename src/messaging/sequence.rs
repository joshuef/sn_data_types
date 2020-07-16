// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{AuthorisationKind, CmdError, DataAuthKind, QueryResponse};
use crate::{
    Error, Sequence as Sequence, SequenceAddress as Address, SequenceEntry as Entry, SequenceIndex as Index,
    SequenceOwner as Owner, SequencePrivPermissions as PrivatePermissions,
    SequencePubPermissions as PublicPermissions, SequenceUser as User, SequenceWriteOp as WriteOp, XorName,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// TODO: docs
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub enum SequenceRead {
    /// Get Sequence from the network.
    Get(Address),
    /// Get a range of entries from an Sequence object on the network.
    GetRange {
        /// Sequence address.
        address: Address,
        /// Range of entries to fetch.
        ///
        /// For example, get 10 last entries:
        /// range: (Index::FromEnd(10), Index::FromEnd(0))
        ///
        /// Get all entries:
        /// range: (Index::FromStart(0), Index::FromEnd(0))
        ///
        /// Get first 5 entries:
        /// range: (Index::FromStart(0), Index::FromStart(5))
        range: (Index, Index),
    },
    /// Get last entry from the Sequence.
    GetLastEntry(Address),
    /// List all current users permissions.
    GetPermissions(Address),
    /// Get current permissions for a specified user(s).
    GetUserPermissions {
        /// Sequence address.
        address: Address,
        /// User to get permissions for.
        user: User,
    },
    /// Get current owner.
    GetOwner(Address),
}

/// TODO: docs
#[allow(clippy::large_enum_variant)]
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub enum SequenceWrite {
    /// Create a new Sequence on the network.
    New(Sequence),
    /// Edit the Sequence (insert/remove entry).
    Edit(WriteOp<Entry>),
    /// Delete a private Sequence.
    ///
    /// This operation MUST return an error if applied to public Sequence. Only the current
    /// owner(s) can perform this action.
    Delete(Address),
    /// Set a new owner. Only the current owner(s) can perform this action.
    SetOwner(WriteOp<Owner>),
    /// Set new permissions for public Sequence.
    SetPubPermissions(WriteOp<PublicPermissions>),
    /// Set new permissions for private Sequence.
    SetPrivPermissions(WriteOp<PrivatePermissions>),
}

impl SequenceRead {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> QueryResponse {
        use SequenceRead::*;
        match *self {
            Get(_) => QueryResponse::GetSequence(Err(error)),
            GetRange { .. } => QueryResponse::GetSequenceRange(Err(error)),
            GetLastEntry(_) => QueryResponse::GetSequenceLastEntry(Err(error)),
            GetPermissions(_) => QueryResponse::GetSequencePermissions(Err(error)),
            GetUserPermissions { .. } => QueryResponse::GetSequenceUserPermissions(Err(error)),
            GetOwner(_) => QueryResponse::GetSequenceOwner(Err(error)),
        }
    }

    /// Returns the access categorisation of the request.
    pub fn authorisation_kind(&self) -> AuthorisationKind {
        use SequenceRead::*;
        match *self {
            Get(address)
            | GetRange { address, .. }
            | GetLastEntry(address)
            | GetPermissions(address)
            | GetUserPermissions { address, .. }
            | GetOwner(address) => {
                if address.is_pub() {
                    AuthorisationKind::Data(DataAuthKind::PublicRead)
                } else {
                    AuthorisationKind::Data(DataAuthKind::PrivateRead)
                }
            }
        }
    }

    /// Returns the address of the destination for request.
    pub fn dst_address(&self) -> XorName {
        use SequenceRead::*;
        match self {
            Get(ref address)
            | GetRange { ref address, .. }
            | GetLastEntry(ref address)
            | GetPermissions(ref address)
            | GetUserPermissions { ref address, .. }
            | GetOwner(ref address) => *address.name(),
        }
    }
}

impl fmt::Debug for SequenceRead {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use SequenceRead::*;
        write!(
            formatter,
            "Request::{}",
            match *self {
                Get(_) => "GetSequence",
                GetRange { .. } => "GetSequenceRange",
                GetLastEntry(_) => "GetSequenceLastEntry",
                GetPermissions { .. } => "GetSequencePermissions",
                GetUserPermissions { .. } => "GetUserPermissions",
                GetOwner { .. } => "GetOwner",
            }
        )
    }
}

impl SequenceWrite {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> CmdError {
        CmdError::Data(error)
    }

    /// Returns the access categorisation of the request.
    pub fn authorisation_kind(&self) -> AuthorisationKind {
        AuthorisationKind::Data(DataAuthKind::Write)
    }

    /// Returns the address of the destination for request.
    pub fn dst_address(&self) -> XorName {
        use SequenceWrite::*;
        match self {
            New(ref data) => *data.name(),
            Delete(ref address) => *address.name(),
            SetPubPermissions(ref op) => *op.address.name(),
            SetPrivPermissions(ref op) => *op.address.name(),
            SetOwner(ref op) => *op.address.name(),
            Edit(ref op) => *op.address.name(),
        }
    }
}

impl fmt::Debug for SequenceWrite {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use SequenceWrite::*;
        write!(
            formatter,
            "Request::{}",
            match *self {
                New(_) => "NewSequence",
                Delete(_) => "DeleteSequence",
                SetPubPermissions(_) => "SetPublicPermissions",
                SetPrivPermissions(_) => "SetPrivatePermissions",
                SetOwner(_) => "SetOwner",
                Edit(_) => "EditSequence",
            }
        )
    }
}
