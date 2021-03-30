use share::ckb_std::error::SysError;
use share::error::HelperError;

/// Error
#[repr(i8)]
#[derive(Debug)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    MissingTypeScript = 5,
    MoreThanOneLiquidityPool,
    MintInitialLiquidityFailed,
    LiquidityArgsUserLockHashMismatch,
    InvalidTypeID,
    VersionDiff = 10,
    SUDTTypeHashMismatch,
    UnknownLiquidity,
    SwapAmountLessThanMin,
    SellSUDTFailed,
    BuySUDTFailed = 15,
    InvalidChangeCell,
    InvalidTotalLiquidity,
    InvalidSwapOutputData,
    BurnLiquidityFailed,
    SUDTGotAmountDiff = 20,
    CKBGotAmountDiff,
    SwapInputSUDTAmountEqZero,
    CKBReserveAmountDiff,
    SUDTReserveAmountDiff,
    InvalidCKBReserve = 25,
    InvalidSUDTReserve,
    CKBInjectAmountDiff,
    SUDTInjectAmountDiff,
    LiquidityPoolTokenDiff,
    InfoLockArgsFrontHalfMismatch = 30,
    InfoLockArgsSecondHalfMismatch,
    InvalidInfoLockCountInCellDeps,
    InvalidInfoLockCountInOutput,
    CellDataLenTooShort,
    InfoCreationCellLockHashMismatch = 35,
    LiquidityArgsInfoTypeHashMismatch,
    InfoCapacityDiff,
    InvalidOutputPoolCapacity,
    InvalidInfoInData,
    LiquiditySUDTTypeHashMismatch = 40,
    AddLiquiditySUDTOutLockHashMismatch,
    InvalidMinCkbInject,
    InvalidMinSUDTInject,
    InvalidMinCkbGot,
    InvalidMinSUDTGot = 45,
    InvalidInfoTypeArgsLen,
    InputCellMoreThanOne,
    AddLiquidityCkbOutLockHashMismatch,
    SUDTCellDataLenTooShort,
    CKBCellDataIsNotEmpty = 50,
    InvalidOutputLockHash,
    RequestCapcityEqSUDTCapcity,
    InvalidOutputTypeHash,
    InvalidSwapOutputCapacity,
    InvalidPoolOutputData = 55,
    NoInfoLockInCellDeps,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

impl From<HelperError> for Error {
    fn from(err: HelperError) -> Self {
        match err {
            HelperError::MissingTypeScript => Self::MissingTypeScript,
        }
    }
}
