export function decode(data) {
    if (!data || data.length < 2) {
        throw new Error("Invalid event data format");
    }

    const [address, raw] = data;

    const bytes = raw.toU8a();
    const operatorHash = bytes.slice(0, 32);    // topic in payload
    const payload = bytes.slice(32);            // event payload

    //console.log(payload);

    const errorMap = [
        "Error::BadOrigin",
        "Error::BankIsClose",
        "Error::BankAccountMaxOut",
        "Error::AccountAlreadyExist",
        "Error::AccountNotFound",
        "Error::AccountBalanceInsufficient",
        "Error::AccountBalanceOverflow",
        "Error::AccountFrozen",
        "Error::LoanComputationOverflow",
        "Error::LoanCollateralInsufficient",
        "Error::LoanAlreadyExist",
        "Error::LoanNotFound",
    ]; 

    const successMap = [
        "Success::BankSetupSuccess",
        "Success::BankCloseSuccess",
        "Success::BankOpenSuccess",
        "Success::AccountDepositSuccess",
        "Success::AccountWithdrawalSuccess",
        "Success::AccountDebitSuccess",
        "Success::AccountCreditSuccess",
        "Success::LoanApplicationSuccess",
        "Success::LoanFullyPaidSuccess",
        "Success::LoanPaymentSuccess",
        "Success::LoanLiquidationSuccess",
    ];     

    //console.log(payload);

    if (payload[1] === 0) {
        return successMap[payload[2]];
    } else if (payload[1] === 1) {
        return errorMap[payload[2]];
    } else {
        throw new Error("Invalid event payload");
    }    
}