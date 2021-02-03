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
    InvalidInfoData,
    SellSUDTFailed,
    BuySUDTFailed = 15,
    InvalidChangeCell,
    InvalidTotalLiquidity,
    InvalidInitialLiquidityTx,
    BurnLiquidityFailed,
    SUDTGotAmountDiff = 20,
    CKBGotAmountDiff,
    InvalidCKBAmount,
    CKBReserveAmountDiff,
    SUDTReserveAmountDiff,
    InvalidCKBReserve = 25,
    InvalidSUDTReserve,
    CKBInjectAmountDiff,
    SUDTInjectAmountDiff,
    LiquidityPoolTokenDiff,
    InfoLockArgsFrontHalfMismatch = 30,
    InfoLockArgsSecondHalfMismatch,
    InfoCreationOutputCellCountMismatch,
    InfoCellHashTypeMismatch,
    CellDataLenTooShort,
    InfoCreationCellLockHashMismatch = 35,
    LiquidityArgsInfoTypeHashMismatch,
    InfoCapacityDiff,
    InAndOutLiquidityDiff,
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
    SwapAmountLessThanMin = 55,
    InvalidSwapOutputData,
    SwapInputSUDTAmountEqZero,
    InvalidOutputPoolData,
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
