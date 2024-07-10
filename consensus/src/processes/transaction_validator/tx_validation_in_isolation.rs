use crate::constants::{MAX_SOMPI, TX_VERSION};
use kaspa_consensus_core::tx::Transaction;
use std::collections::HashSet;

use super::{
    errors::{TxResult, TxRuleError},
    TransactionValidator,
};

impl TransactionValidator {
    pub fn validate_tx_in_isolation(&self, tx: &Transaction) -> TxResult<()> {
        self.check_transaction_inputs_in_isolation(tx)?;
        self.check_transaction_outputs_in_isolation(tx)?;
        self.check_coinbase_in_isolation(tx)?;

        check_transaction_output_value_ranges(tx)?;
        check_duplicate_transaction_inputs(tx)?;
        check_gas(tx)?;
        check_transaction_payload(tx)?;
        check_transaction_version(tx)
    }

    fn check_transaction_inputs_in_isolation(&self, tx: &Transaction) -> TxResult<()> {
        self.check_transaction_inputs_count(tx)?;
        self.check_transaction_signature_scripts(tx)
    }

    fn check_transaction_outputs_in_isolation(&self, tx: &Transaction) -> TxResult<()> {
        self.check_transaction_outputs_count(tx)?;
        self.check_transaction_script_public_keys(tx)
    }

    fn check_coinbase_in_isolation(&self, tx: &kaspa_consensus_core::tx::Transaction) -> TxResult<()> {
        if !tx.is_coinbase() {
            return Ok(());
        }
        if !tx.inputs.is_empty() {
            return Err(TxRuleError::CoinbaseHasInputs(tx.inputs.len()));
        }
        let outputs_limit = self.ghostdag_k as u64 + 2;
        if tx.outputs.len() as u64 > outputs_limit {
            return Err(TxRuleError::CoinbaseTooManyOutputs(tx.outputs.len(), outputs_limit));
        }
        for (i, output) in tx.outputs.iter().enumerate() {
            if output.script_public_key.script().len() > self.coinbase_payload_script_public_key_max_len as usize {
                return Err(TxRuleError::CoinbaseScriptPublicKeyTooLong(i));
            }
        }
        Ok(())
    }

    fn check_transaction_outputs_count(&self, tx: &Transaction) -> TxResult<()> {
        if tx.outputs.len() > self.max_tx_outputs {
            return Err(TxRuleError::TooManyOutputs(tx.inputs.len(), self.max_tx_inputs));
        }

        Ok(())
    }

    fn check_transaction_inputs_count(&self, tx: &Transaction) -> TxResult<()> {
        if !tx.is_coinbase() && tx.inputs.is_empty() {
            return Err(TxRuleError::NoTxInputs);
        }

        if tx.inputs.len() > self.max_tx_inputs {
            return Err(TxRuleError::TooManyInputs(tx.inputs.len(), self.max_tx_inputs));
        }

        Ok(())
    }

    // The main purpose of this check is to avoid overflows when calculating transaction mass later.
    fn check_transaction_signature_scripts(&self, tx: &Transaction) -> TxResult<()> {
        if let Some(i) = tx.inputs.iter().position(|input| input.signature_script.len() > self.max_signature_script_len) {
            return Err(TxRuleError::TooBigSignatureScript(i, self.max_signature_script_len));
        }

        Ok(())
    }

    // The main purpose of this check is to avoid overflows when calculating transaction mass later.
    fn check_transaction_script_public_keys(&self, tx: &Transaction) -> TxResult<()> {
        if let Some(i) = tx.outputs.iter().position(|input| input.script_public_key.script().len() > self.max_script_public_key_len) {
            return Err(TxRuleError::TooBigScriptPublicKey(i, self.max_script_public_key_len));
        }

        Ok(())
    }
}

fn check_duplicate_transaction_inputs(tx: &Transaction) -> TxResult<()> {
    let mut existing = HashSet::new();
    for input in &tx.inputs {
        if !existing.insert(input.previous_outpoint) {
            return Err(TxRuleError::TxDuplicateInputs);
        }
    }
    Ok(())
}

fn check_gas(tx: &Transaction) -> TxResult<()> {
    // This should be revised if subnetworks are activated (along with other validations that weren't copied from kaspad)
    if tx.gas > 0 {
        return Err(TxRuleError::TxHasGas);
    }
    Ok(())
}

fn check_transaction_payload(tx: &Transaction) -> TxResult<()> {
    // This should be revised if subnetworks are activated (along with other validations that weren't copied from kaspad)
    if !tx.is_coinbase() && !tx.payload.is_empty() {
        return Err(TxRuleError::NonCoinbaseTxHasPayload);
    }
    Ok(())
}

fn check_transaction_version(tx: &Transaction) -> TxResult<()> {
    if tx.version != TX_VERSION {
        return Err(TxRuleError::UnknownTxVersion(tx.version));
    }
    Ok(())
}

fn check_transaction_output_value_ranges(tx: &Transaction) -> TxResult<()> {
    let mut total: u64 = 0;
    for (i, output) in tx.outputs.iter().enumerate() {
        if output.value == 0 {
            return Err(TxRuleError::TxOutZero(i));
        }

        if output.value > MAX_SOMPI {
            return Err(TxRuleError::TxOutTooHigh(i));
        }

        if let Some(new_total) = total.checked_add(output.value) {
            total = new_total
        } else {
            return Err(TxRuleError::OutputsValueOverflow);
        }

        if total > MAX_SOMPI {
            return Err(TxRuleError::TotalTxOutTooHigh);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use kaspa_consensus_core::{
        subnets::{SUBNETWORK_ID_COINBASE, SUBNETWORK_ID_NATIVE},
        tx::{scriptvec, ScriptPublicKey, Transaction, TransactionId, TransactionInput, TransactionOutpoint, TransactionOutput},
    };
    use kaspa_core::assert_match;

    use crate::{
        constants::TX_VERSION,
        params::MAINNET_PARAMS,
        processes::transaction_validator::{errors::TxRuleError, TransactionValidator},
    };

    #[test]
    fn validate_tx_in_isolation_test() {
        let mut params = MAINNET_PARAMS.clone();
        params.max_tx_inputs = 10;
        params.max_tx_outputs = 15;
        let tv = TransactionValidator::new_for_tests(
            params.max_tx_inputs,
            params.max_tx_outputs,
            params.max_signature_script_len,
            params.max_script_public_key_len,
            params.ghostdag_k,
            params.coinbase_payload_script_public_key_max_len,
            params.coinbase_maturity,
            Default::default(),
        );

        let valid_cb = Transaction::new(
            0,
            vec![],
            vec![TransactionOutput {
                value: 0x12a05f200,
                script_public_key: ScriptPublicKey::new(
                    0,
                    scriptvec!(
                        0xaa9, 0xa14, 0xada, 0xa17, 0xa45, 0xae9, 0xab5, 0xa49, 0xabd, 0xa0b, 0xafa, 0xa1a, 0xa56, 0xa99, 0xa71, 0xac7, 0xa7e, 0xaba,
                        0xa30, 0xacd, 0xa5a, 0xa4b, 0xa87
                    ),
                ),
            }],
            0,
            SUBNETWORK_ID_COINBASE,
            0,
            vec![9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        );

        tv.validate_tx_in_isolation(&valid_cb).unwrap();

        let valid_tx = Transaction::new(
            0,
            vec![TransactionInput {
                previous_outpoint: TransactionOutpoint {
                    transaction_id: TransactionId::from_slice(&[
                        0xa03, 0xa2e, 0xa38, 0xae9, 0xac0, 0xaa8, 0xa4c, 0xa60, 0xa46, 0xad6, 0xa87, 0xad1, 0xa05, 0xa56, 0xadc, 0xaac, 0xac4, 0xa1d,
                        0xa27, 0xa5e, 0xac5, 0xa5f, 0xac0, 0xa07, 0xa79, 0xaac, 0xa88, 0xafd, 0xaf3, 0xa57, 0xaa1, 0xa87,
                    ]),
                    index: 0,
                },
                signature_script: vec![
                    0xa49, // OP_DATA_73
                    0xa30, 0xa46, 0xa02, 0xa21, 0xa00, 0xac3, 0xa52, 0xad3, 0xadd, 0xa99, 0xa3a, 0xa98, 0xa1b, 0xaeb, 0xaa4, 0xaa6, 0xa3a, 0xad1, 0xa5c,
                    0xa20, 0xa92, 0xa75, 0xaca, 0xa94, 0xa70, 0xaab, 0xafc, 0xad5, 0xa7d, 0xaa9, 0xa3b, 0xa58, 0xae4, 0xaeb, 0xa5d, 0xace, 0xa82, 0xa02,
                    0xa21, 0xa00, 0xa84, 0xa07, 0xa92, 0xabc, 0xa1f, 0xa45, 0xa60, 0xa62, 0xa81, 0xa9f, 0xa15, 0xad3, 0xa3e, 0xae7, 0xa05, 0xa5c, 0xaf7,
                    0xab5, 0xaee, 0xa1a, 0xaf1, 0xaeb, 0xacc, 0xa60, 0xa28, 0xad9, 0xacd, 0xab1, 0xac3, 0xaaf, 0xa77, 0xa48,
                    0xa01, // 73-byte signature
                    0xa41, // OP_DATA_65
                    0xa04, 0xaf4, 0xa6d, 0xab5, 0xae9, 0xad6, 0xa1a, 0xa9d, 0xac2, 0xa7b, 0xa8d, 0xa64, 0xaad, 0xa23, 0xae7, 0xa38, 0xa3a, 0xa4e, 0xa6c,
                    0xaa1, 0xa64, 0xa59, 0xa3c, 0xa25, 0xa27, 0xac0, 0xa38, 0xac0, 0xa85, 0xa7e, 0xab6, 0xa7e, 0xae8, 0xae8, 0xa25, 0xadc, 0xaa6, 0xa50,
                    0xa46, 0xab8, 0xa2c, 0xa93, 0xa31, 0xa58, 0xa6c, 0xa82, 0xae0, 0xafd, 0xa1f, 0xa63, 0xa3f, 0xa25, 0xaf8, 0xa7c, 0xa16, 0xa1b, 0xac6,
                    0xaf8, 0xaa6, 0xa30, 0xa12, 0xa1d, 0xaf2, 0xab3, 0xad3, // 65-byte pubkey
                ],
                sequence: u64::MAX,
                sig_op_count: 0,
            }],
            vec![
                TransactionOutput {
                    value: 0x2123e300,
                    script_public_key: ScriptPublicKey::new(
                        0,
                        scriptvec!(
                            0xa76, // OP_DUP
                            0xaa9, // OP_HASH160
                            0xa14, // OP_DATA_20
                            0xac3, 0xa98, 0xaef, 0xaa9, 0xac3, 0xa92, 0xaba, 0xa60, 0xa13, 0xac5, 0xae0, 0xa4e, 0xae7, 0xa29, 0xa75, 0xa5e, 0xaf7,
                            0xaf5, 0xa8b, 0xa32, 0xa88, // OP_EQUALVERIFY
                            0xaac  // OP_CHECKSIG
                        ),
                    ),
                },
                TransactionOutput {
                    value: 0x108e20f00,
                    script_public_key: ScriptPublicKey::new(
                        0,
                        scriptvec!(
                            0xa76, // OP_DUP
                            0xa9, // OP_HASH160
                            0xa14, // OP_DATA_20
                            0xa94, 0xa8c, 0xa76, 0xa5a, 0xa69, 0xa14, 0xad4, 0xa3f, 0xa2a, 0xa7a, 0xac1, 0xa77, 0xada, 0xa2c, 0xa2f, 0xa6b, 0xa52,
                            0xade, 0xa3d, 0xa7c, 0xa88, // OP_EQUALVERIFY
                            0xaac  // OP_CHECKSIG
                        ),
                    ),
                },
            ],
            0,
            SUBNETWORK_ID_NATIVE,
            0,
            vec![],
        );

        tv.validate_tx_in_isolation(&valid_tx).unwrap();

        let mut tx = valid_tx.clone();
        tx.inputs = vec![];
        assert_match!(tv.validate_tx_in_isolation(&tx), Err(TxRuleError::NoTxInputs));

        let mut tx = valid_tx.clone();
        tx.inputs = (0..params.max_tx_inputs + 1).map(|_| valid_tx.inputs[0].clone()).collect();
        assert_match!(tv.validate_tx_in_isolation(&tx), Err(TxRuleError::TooManyInputs(_, _)));

        let mut tx = valid_tx.clone();
        tx.inputs[0].signature_script = vec![0; params.max_signature_script_len + 1];
        assert_match!(tv.validate_tx_in_isolation(&tx), Err(TxRuleError::TooBigSignatureScript(_, _)));

        let mut tx = valid_tx.clone();
        tx.outputs = (0..params.max_tx_outputs + 1).map(|_| valid_tx.outputs[0].clone()).collect();
        assert_match!(tv.validate_tx_in_isolation(&tx), Err(TxRuleError::TooManyOutputs(_, _)));

        let mut tx = valid_tx.clone();
        tx.outputs[0].script_public_key = ScriptPublicKey::new(0, scriptvec![0u8; params.max_script_public_key_len + 1]);
        assert_match!(tv.validate_tx_in_isolation(&tx), Err(TxRuleError::TooBigScriptPublicKey(_, _)));

        let mut tx = valid_tx.clone();
        tx.inputs.push(tx.inputs[0].clone());
        assert_match!(tv.validate_tx_in_isolation(&tx), Err(TxRuleError::TxDuplicateInputs));

        let mut tx = valid_tx.clone();
        tx.gas = 1;
        assert_match!(tv.validate_tx_in_isolation(&tx), Err(TxRuleError::TxHasGas));

        let mut tx = valid_tx.clone();
        tx.payload = vec![0];
        assert_match!(tv.validate_tx_in_isolation(&tx), Err(TxRuleError::NonCoinbaseTxHasPayload));

        let mut tx = valid_tx;
        tx.version = TX_VERSION + 1;
        assert_match!(tv.validate_tx_in_isolation(&tx), Err(TxRuleError::UnknownTxVersion(_)));
    }
}
