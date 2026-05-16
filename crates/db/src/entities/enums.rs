use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use ts_rs::TS;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(
    export,
    export_to = "../../../packages/types/src/db/TransactionDirection.ts"
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionDirection {
    #[sea_orm(string_value = "IN")]
    In,
    #[sea_orm(string_value = "OUT")]
    Out,
}

impl TransactionDirection {
    #[must_use]
    pub fn counterparty_role(&self) -> TxnPartyRole {
        match self {
            Self::In => TxnPartyRole::Sender,
            Self::Out => TxnPartyRole::Receiver,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(
    export,
    export_to = "../../../packages/types/src/db/TransactionSource.ts"
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionSource {
    #[sea_orm(string_value = "MANUAL")]
    Manual,
    #[sea_orm(string_value = "OCR")]
    Ocr,
    #[sea_orm(string_value = "STATEMENT")]
    Statement,
    #[sea_orm(string_value = "P2P")]
    P2p,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(
    export,
    export_to = "../../../packages/types/src/db/TransactionStatus.ts"
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    #[sea_orm(string_value = "COMPLETED")]
    Completed,
    #[sea_orm(string_value = "PENDING")]
    Pending,
    #[sea_orm(string_value = "CANCELLED")]
    Cancelled,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(export, export_to = "../../../packages/types/src/db/IdentifierType.ts")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IdentifierType {
    #[sea_orm(string_value = "UPI")]
    Upi,
    #[sea_orm(string_value = "PHONE")]
    Phone,
    #[sea_orm(string_value = "BANK_ACC")]
    BankAcc,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(export, export_to = "../../../packages/types/src/db/TxnPartyRole.ts")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TxnPartyRole {
    #[sea_orm(string_value = "SENDER")]
    Sender,
    #[sea_orm(string_value = "RECEIVER")]
    Receiver,
    #[sea_orm(string_value = "COUNTERPARTY")]
    Counterparty,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(
    export,
    export_to = "../../../packages/types/src/db/SubscriptionCycle.ts"
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubscriptionCycle {
    #[sea_orm(string_value = "WEEKLY")]
    Weekly,
    #[sea_orm(string_value = "MONTHLY")]
    Monthly,
    #[sea_orm(string_value = "YEARLY")]
    Yearly,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(export, export_to = "../../../packages/types/src/db/AlertChannel.ts")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlertChannel {
    #[sea_orm(string_value = "EMAIL")]
    Email,
    #[sea_orm(string_value = "PUSH")]
    Push,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(
    export,
    export_to = "../../../packages/types/src/db/P2pRequestStatus.ts"
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum P2pRequestStatus {
    #[sea_orm(string_value = "PENDING")]
    Pending,
    #[sea_orm(string_value = "MAPPED")]
    Mapped,
    #[sea_orm(string_value = "REJECTED")]
    Rejected,
    #[sea_orm(string_value = "APPROVED")]
    Approved,
    #[sea_orm(string_value = "GROUP_INVITE")]
    GroupInvite,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(export, export_to = "../../../packages/types/src/db/GroupRole.ts")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GroupRole {
    #[sea_orm(string_value = "ADMIN")]
    Admin,
    #[sea_orm(string_value = "MEMBER")]
    Member,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(export, export_to = "../../../packages/types/src/db/WalletType.ts")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WalletType {
    #[sea_orm(string_value = "CASH")]
    Cash,
    #[sea_orm(string_value = "BANK")]
    Bank,
    #[sea_orm(string_value = "CREDIT_CARD")]
    CreditCard,
    #[sea_orm(string_value = "UPI_WALLET")]
    UpiWallet,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(export, export_to = "../../../packages/types/src/db/LedgerTabType.ts")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LedgerTabType {
    #[sea_orm(string_value = "LENT")]
    Lent,
    #[sea_orm(string_value = "BORROWED")]
    Borrowed,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(export, export_to = "../../../packages/types/src/db/BudgetPeriod.ts")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BudgetPeriod {
    #[sea_orm(string_value = "WEEKLY")]
    Weekly,
    #[sea_orm(string_value = "MONTHLY")]
    Monthly,
    #[sea_orm(string_value = "YEARLY")]
    Yearly,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(
    export,
    export_to = "../../../packages/types/src/db/LedgerTabStatus.ts"
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LedgerTabStatus {
    #[sea_orm(string_value = "OPEN")]
    Open,
    #[sea_orm(string_value = "PARTIALLY_PAID")]
    PartiallyPaid,
    #[sea_orm(string_value = "SETTLED")]
    Settled,
}
