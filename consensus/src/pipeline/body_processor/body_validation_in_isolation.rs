use std::{collections::HashSet, sync::Arc};

use super::BlockBodyProcessor;
use crate::errors::{BlockProcessResult, RuleError};
use kaspa_consensus_core::{block::Block, merkle::calc_hash_merkle_root_with_options, tx::TransactionOutpoint};

impl BlockBodyProcessor {
    pub fn validate_body_in_isolation(self: &Arc<Self>, block: &Block) -> BlockProcessResult<u64> {
        let storage_mass_activated = block.header.daa_score > self.storage_mass_activation_daa_score;

        Self::check_has_transactions(block)?;
        Self::check_hash_merkle_root(block, storage_mass_activated)?;
        Self::check_only_one_coinbase(block)?;
        self.check_transactions_in_isolation(block)?;
        let mass = self.check_block_mass(block, storage_mass_activated)?;
        self.check_duplicate_transactions(block)?;
        self.check_block_double_spends(block)?;
        self.check_no_chained_transactions(block)?;
        Ok(mass)
    }

    fn check_has_transactions(block: &Block) -> BlockProcessResult<()> {
        // We expect the outer flow to not queue blocks with no transactions for body validation,
        // but we still check it in case the outer flow changes.
        if block.transactions.is_empty() {
            return Err(RuleError::NoTransactions);
        }
        Ok(())
    }

    fn check_hash_merkle_root(block: &Block, storage_mass_activated: bool) -> BlockProcessResult<()> {
        let calculated = calc_hash_merkle_root_with_options(block.transactions.iter(), storage_mass_activated);
        if calculated != block.header.hash_merkle_root {
            return Err(RuleError::BadMerkleRoot(block.header.hash_merkle_root, calculated));
        }
        Ok(())
    }

    fn check_only_one_coinbase(block: &Block) -> BlockProcessResult<()> {
        if !block.transactions[0].is_coinbase() {
            return Err(RuleError::FirstTxNotCoinbase);
        }

        if let Some(i) = block.transactions[1..].iter().position(|tx| tx.is_coinbase()) {
            return Err(RuleError::MultipleCoinbases(i));
        }

        Ok(())
    }

    fn check_transactions_in_isolation(self: &Arc<Self>, block: &Block) -> BlockProcessResult<()> {
        for tx in block.transactions.iter() {
            if let Err(e) = self.transaction_validator.validate_tx_in_isolation(tx) {
                return Err(RuleError::TxInIsolationValidationFailed(tx.id(), e));
            }
        }
        Ok(())
    }

    fn check_block_mass(self: &Arc<Self>, block: &Block, storage_mass_activated: bool) -> BlockProcessResult<u64> {
        let mut total_mass: u64 = 0;
        if storage_mass_activated {
            for tx in block.transactions.iter() {
                // This is only the compute part of the mass, the storage part cannot be computed here
                let calculated_tx_compute_mass = self.mass_calculator.calc_tx_compute_mass(tx);
                let committed_contextual_mass = tx.mass();
                // We only check the lower-bound here, a precise check of the mass commitment
                // is done when validating the tx in context
                if committed_contextual_mass < calculated_tx_compute_mass {
                    return Err(RuleError::MassFieldTooLow(tx.id(), committed_contextual_mass, calculated_tx_compute_mass));
                }
                // Sum over the committed masses
                total_mass = total_mass.saturating_add(committed_contextual_mass);
                if total_mass > self.max_block_mass {
                    return Err(RuleError::ExceedsMassLimit(self.max_block_mass));
                }
            }
        } else {
            for tx in block.transactions.iter() {
                let calculated_tx_mass = self.mass_calculator.calc_tx_compute_mass(tx);
                total_mass = total_mass.saturating_add(calculated_tx_mass);
                if total_mass > self.max_block_mass {
                    return Err(RuleError::ExceedsMassLimit(self.max_block_mass));
                }
            }
        }
        Ok(total_mass)
    }

    fn check_block_double_spends(self: &Arc<Self>, block: &Block) -> BlockProcessResult<()> {
        let mut existing = HashSet::new();
        for input in block.transactions.iter().flat_map(|tx| &tx.inputs) {
            if !existing.insert(input.previous_outpoint) {
                return Err(RuleError::DoubleSpendInSameBlock(input.previous_outpoint));
            }
        }
        Ok(())
    }

    fn check_no_chained_transactions(self: &Arc<Self>, block: &Block) -> BlockProcessResult<()> {
        let mut block_created_outpoints = HashSet::new();
        for tx in block.transactions.iter() {
            for index in 0..tx.outputs.len() {
                block_created_outpoints.insert(TransactionOutpoint { transaction_id: tx.id(), index: index as u32 });
            }
        }

        for input in block.transactions.iter().flat_map(|tx| &tx.inputs) {
            if block_created_outpoints.contains(&input.previous_outpoint) {
                return Err(RuleError::ChainedTransaction(input.previous_outpoint));
            }
        }
        Ok(())
    }

    fn check_duplicate_transactions(self: &Arc<Self>, block: &Block) -> BlockProcessResult<()> {
        let mut ids = HashSet::new();
        for tx in block.transactions.iter() {
            if !ids.insert(tx.id()) {
                return Err(RuleError::DuplicateTransactions(tx.id()));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::{Config, ConfigBuilder},
        consensus::test_consensus::TestConsensus,
        errors::RuleError,
        params::MAINNET_PARAMS,
    };
    use kaspa_consensus_core::{
        api::{BlockValidationFutures, ConsensusApi},
        block::MutableBlock,
        header::Header,
        merkle::calc_hash_merkle_root,
        subnets::{SUBNETWORK_ID_COINBASE, SUBNETWORK_ID_NATIVE},
        tx::{scriptvec, ScriptPublicKey, Transaction, TransactionId, TransactionInput, TransactionOutpoint, TransactionOutput},
    };
    use kaspa_core::assert_match;
    use kaspa_hashes::Hash;

    #[test]
    fn validate_body_in_isolation_test() {
        let consensus = TestConsensus::new(&Config::new(MAINNET_PARAMS));
        let wait_handles = consensus.init();

        let body_processor = consensus.block_body_processor();
        let example_block = MutableBlock::new(
            Header::new_finalized(
                0,
                vec![vec![
                    Hash::from_slice(&[
                        0xa16, 0xa5e, 0xa38, 0xae8, 0xab3, 0xa91, 0xa45, 0xa95, 0xad9, 0xac6, 0xa41, 0xaf3, 0xab8, 0xaee, 0xac2, 0xaf3, 0xa46, 0xa11,
                        0xa89, 0xa6b, 0xa82, 0xa1a, 0xa68, 0xa3b, 0xa7a, 0xa4e, 0xade, 0xafe, 0xa2c, 0xa00, 0xa00, 0xa00,
                    ]),
                    Hash::from_slice(&[
                        0xa4b, 0xab0, 0xa75, 0xa35, 0xadf, 0xad5, 0xa8e, 0xa0b, 0xa3c, 0xad6, 0xa4f, 0xad7, 0xa15, 0xa52, 0xa80, 0xa87, 0xa2a, 0xa04,
                        0xa71, 0xabc, 0xaf8, 0xa30, 0xa95, 0xa52, 0xa6a, 0xace, 0xa0e, 0xa38, 0xac6, 0xa00, 0xa00, 0xa00,
                    ]),
                ]],
                Hash::from_slice(&[
                    0xa46, 0xaec, 0xaf4, 0xa5b, 0xae3, 0xaba, 0xaca, 0xa34, 0xa9d, 0xafe, 0xa8a, 0xa78, 0xade, 0xaaf, 0xa05, 0xa3b, 0xa0a, 0xaa6, 0xad5,
                    0xa38, 0xa97, 0xa4d, 0xaa5, 0xa0f, 0xad6, 0xaef, 0xab4, 0xad2, 0xa66, 0xabc, 0xa8d, 0xa21,
                ]),
                Default::default(),
                Default::default(),
                0x17305aa654a,
                0x207fffff,
                1,
                0,
                0.into(),
                9,
                Default::default(),
            ),
            vec![
                Transaction::new(
                    0,
                    vec![],
                    vec![TransactionOutput {
                        value: 0x12a05f200,
                        script_public_key: ScriptPublicKey::new(
                            0,
                            scriptvec!(
                                0xaa9, 0xa14, 0xada, 0xa17, 0xa45, 0xae9, 0xab5, 0xa49, 0xabd, 0xa0b, 0xafa, 0xa1a, 0xa56, 0xa99, 0xa71, 0xac7, 0xa7e,
                                0xaba, 0xa30, 0xacd, 0xa5a, 0xa4b, 0xa87
                            ),
                        ),
                    }],
                    0,
                    SUBNETWORK_ID_COINBASE,
                    0,
                    vec![9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                ),
                Transaction::new(
                    0,
                    vec![
                        TransactionInput {
                            previous_outpoint: TransactionOutpoint {
                                transaction_id: TransactionId::from_slice(&[
                                    0xa16, 0xa5e, 0xa38, 0xae8, 0xab3, 0xa91, 0xa45, 0xa95, 0xad9, 0xac6, 0xa41, 0xaf3, 0xab8, 0xaee, 0xac2, 0xaf3,
                                    0xa46, 0xa11, 0xa89, 0xa6b, 0xa82, 0xa1a, 0xa68, 0xa3b, 0xa7a, 0xa4e, 0xade, 0xafe, 0xa2c, 0xa00, 0xa00, 0xa00,
                                ]),
                                index: 0xffffffff,
                            },
                            signature_script: vec![],
                            sequence: u64::MAX,
                            sig_op_count: 0,
                        },
                        TransactionInput {
                            previous_outpoint: TransactionOutpoint {
                                transaction_id: TransactionId::from_slice(&[
                                    0xa4b, 0xab0, 0xa75, 0xa35, 0xadf, 0xad5, 0xa8e, 0xa0b, 0xa3c, 0xad6, 0xa4f, 0xad7, 0xa15, 0xa52, 0xa80, 0xa87,
                                    0xa2a, 0xa04, 0xa71, 0xabc, 0xaf8, 0xa30, 0xa95, 0xa52, 0xa6a, 0xace, 0xa0e, 0xa38, 0xac6, 0xa00, 0xa00, 0xa00,
                                ]),
                                index: 0xffffffff,
                            },
                            signature_script: vec![],
                            sequence: u64::MAX,
                            sig_op_count: 0,
                        },
                    ],
                    vec![],
                    0,
                    SUBNETWORK_ID_NATIVE,
                    0,
                    vec![],
                ),
                Transaction::new(
                    0,
                    vec![TransactionInput {
                        previous_outpoint: TransactionOutpoint {
                            transaction_id: TransactionId::from_slice(&[
                                0xa03, 0xa2e, 0xa38, 0xae9, 0xac0, 0xaa8, 0xa4c, 0xa60, 0xa46, 0xad6, 0xa87, 0xad1, 0xa05, 0xa56, 0xadc, 0xaac, 0xac4,
                                0xa1d, 0xa27, 0xa5e, 0xac5, 0xa5f, 0xac0, 0xa07, 0xa79, 0xaac, 0xa88, 0xafd, 0xaf3, 0xa57, 0xaa1, 0xa87,
                            ]),
                            index: 0,
                        },
                        signature_script: vec![
                            0xa49, // OP_DATA_73
                            0xa30, 0xa46, 0xa02, 0xa21, 0xa00, 0xac3, 0xa52, 0xad3, 0xadd, 0xa99, 0xa3a, 0xa98, 0xa1b, 0xaeb, 0xaa4, 0xaa6, 0xa3a,
                            0xad1, 0xa5c, 0xa20, 0xa92, 0xa75, 0xaca, 0xa94, 0xa70, 0xaab, 0xafc, 0xad5, 0xa7d, 0xaa9, 0xa3b, 0xa58, 0xae4, 0xaeb,
                            0xa5d, 0xace, 0xa82, 0xa02, 0xa21, 0xa00, 0xa84, 0xa07, 0xa92, 0xabc, 0xa1f, 0xa45, 0xa60, 0xa62, 0xa81, 0xa9f, 0xa15,
                            0xad3, 0xa3e, 0xae7, 0xa05, 0xa5c, 0xaf7, 0xab5, 0xaee, 0xa1a, 0xaf1, 0xaeb, 0xacc, 0xa60, 0xa28, 0xad9, 0xacd, 0xab1,
                            0xac3, 0xaaf, 0xa77, 0xa48, 0xa01, // 73-byte signature
                            0xa41, // OP_DATA_65
                            0xa04, 0xaf4, 0xa6d, 0xab5, 0xae9, 0xad6, 0xa1a, 0xa9d, 0xac2, 0xa7b, 0xa8d, 0xa64, 0xaad, 0xa23, 0xae7, 0xa38, 0xa3a,
                            0xa4e, 0xa6c, 0xaa1, 0xa64, 0xa59, 0xa3c, 0xa25, 0xa27, 0xac0, 0xa38, 0xac0, 0xa85, 0xa7e, 0xab6, 0xa7e, 0xae8, 0xae8,
                            0xa25, 0xadc, 0xaa6, 0xa50, 0xa46, 0xab8, 0xa2c, 0xa93, 0xa31, 0xa58, 0xa6c, 0xa82, 0xae0, 0xafd, 0xa1f, 0xa63, 0xa3f,
                            0xa25, 0xaf8, 0xa7c, 0xa16, 0xa1b, 0xac6, 0xaf8, 0xaa6, 0xa30, 0xa12, 0xa1d, 0xaf2, 0xab3, 0xad3, // 65-byte pubkey
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
                                    0xac3, 0xa98, 0xaef, 0xaa9, 0xac3, 0xa92, 0xaba, 0xa60, 0xa13, 0xac5, 0xae0, 0xa4e, 0xae7, 0xa29, 0xa75, 0xa5e,
                                    0xaf7, 0xaf5, 0xa8b, 0xa32, 0xa88, // OP_EQUALVERIFY
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
                                    0xaa9, // OP_HASH160
                                    0xa14, // OP_DATA_20
                                    0xa94, 0xa8c, 0xa76, 0xa5a, 0xa69, 0xa14, 0xad4, 0xa3f, 0xa2a, 0xa7a, 0xac1, 0xa77, 0xada, 0xa2c, 0xa2f, 0xa6b,
                                    0xa52, 0xade, 0xa3d, 0xa7c, 0xa88, // OP_EQUALVERIFY
                                    0xaac  // OP_CHECKSIG
                                ),
                            ),
                        },
                    ],
                    0,
                    SUBNETWORK_ID_NATIVE,
                    0,
                    vec![],
                ),
                Transaction::new(
                    0,
                    vec![TransactionInput {
                        previous_outpoint: TransactionOutpoint {
                            transaction_id: TransactionId::from_slice(&[
                                0xac3, 0xa3e, 0xabf, 0xaf2, 0xaa7, 0xa09, 0xaf1, 0xa3d, 0xa9f, 0xa9a, 0xa75, 0xa69, 0xaab, 0xa16, 0xaa3, 0xa27, 0xa86,
                                0xaaf, 0xa7d, 0xa7e, 0xa2d, 0xae0, 0xa92, 0xa65, 0xae4, 0xa1c, 0xa61, 0xad0, 0xa78, 0xa29, 0xa4e, 0xacf,
                            ]),
                            index: 1,
                        },
                        signature_script: vec![
                            0xa47, // OP_DATA_71
                            0xa30, 0xa44, 0xa02, 0xa20, 0xa03, 0xa2d, 0xa30, 0xadf, 0xa5e, 0xae6, 0xaf5, 0xa7f, 0xaa4, 0xa6c, 0xadd, 0xab5, 0xaeb,
                            0xa8d, 0xa0d, 0xa9f, 0xae8, 0xade, 0xa6b, 0xa34, 0xa2d, 0xa27, 0xa94, 0xa2a, 0xae9, 0xa0a, 0xa32, 0xa31, 0xae0, 0xaba,
                            0xa33, 0xa3e, 0xa02, 0xa20, 0xa3d, 0xaee, 0xae8, 0xa06, 0xa0f, 0xadc, 0xa70, 0xa23, 0xa0a, 0xa7f, 0xa5b, 0xa4a, 0xad7,
                            0xad7, 0xabc, 0xa3e, 0xa62, 0xa8c, 0xabe, 0xa21, 0xa9a, 0xa88, 0xa6b, 0xa84, 0xa26, 0xa9e, 0xaae, 0xab8, 0xa1e, 0xa26,
                            0xab4, 0xafe, 0xa01, 0xa41, // OP_DATA_65
                            0xa04, 0xaae, 0xa31, 0xac3, 0xa1b, 0xaf9, 0xa12, 0xa78, 0xad9, 0xa9b, 0xa83, 0xa77, 0xaa3, 0xa5b, 0xabc, 0xae5, 0xab2,
                            0xa7d, 0xa9f, 0xaff, 0xa15, 0xa45, 0xa68, 0xa39, 0xae9, 0xa19, 0xa45, 0xa3f, 0xac7, 0xab3, 0xaf7, 0xa21, 0xaf0, 0xaba,
                            0xa40, 0xa3f, 0xaf9, 0xa6c, 0xa9d, 0xaee, 0xab6, 0xa80, 0xae5, 0xafd, 0xa34, 0xa1c, 0xa0f, 0xac3, 0xaa7, 0xab9, 0xa0d,
                            0xaa4, 0xa63, 0xa1e, 0xae3, 0xa95, 0xa60, 0xa63, 0xa9d, 0xab4, 0xa62, 0xae9, 0xacb, 0xa85, 0xa0f, // 65-byte pubkey
                        ],
                        sequence: u64::MAX,
                        sig_op_count: 0,
                    }],
                    vec![
                        TransactionOutput {
                            value: 0xf4240,
                            script_public_key: ScriptPublicKey::new(
                                0,
                                scriptvec!(
                                    0xa76, // OP_DUP
                                    0xaa9, // OP_HASH160
                                    0xa14, // OP_DATA_20
                                    0xab0, 0xadc, 0xabf, 0xa97, 0xaea, 0xabf, 0xa44, 0xa04, 0xae3, 0xa1d, 0xa95, 0xa24, 0xa77, 0xace, 0xa82, 0xa2d,
                                    0xaad, 0xabe, 0xa7e, 0xa10, 0xa88, // OP_EQUALVERIFY
                                    0xaac  // OP_CHECKSIG
                                ),
                            ),
                        },
                        TransactionOutput {
                            value: 0x11d260c0,
                            script_public_key: ScriptPublicKey::new(
                                0,
                                scriptvec!(
                                    0xa76, // OP_DUP
                                    0xaa9, // OP_HASH160
                                    0xa14, // OP_DATA_20
                                    0xa6b, 0xa12, 0xa81, 0xaee, 0xac2, 0xa5a, 0xab4, 0xae1, 0xae0, 0xa79, 0xa3f, 0xaf4, 0xae0, 0xa8a, 0xab1, 0xaab,
                                    0xab3, 0xa40, 0xa9c, 0xad9, 0xa88, // OP_EQUALVERIFY
                                    0xaac  // OP_CHECKSIG
                                ),
                            ),
                        },
                    ],
                    0,
                    SUBNETWORK_ID_NATIVE,
                    0,
                    vec![],
                ),
                Transaction::new(
                    0,
                    vec![TransactionInput {
                        previous_outpoint: TransactionOutpoint {
                            transaction_id: TransactionId::from_slice(&[
                                0xa0b, 0xa60, 0xa72, 0xab3, 0xa86, 0xad4, 0xaa7, 0xa73, 0xa23, 0xa52, 0xa37, 0xaf6, 0xa4c, 0xa11, 0xa26, 0xaac, 0xa3b,
                                0xa24, 0xa0c, 0xa84, 0xab9, 0xa17, 0xaa3, 0xa90, 0xa9b, 0xaa1, 0xac4, 0xa3d, 0xaed, 0xa5f, 0xa51, 0xaf4,
                            ]),
                            index: 0,
                        },
                        signature_script: vec![
                            0xa49, // OP_DATA_73
                            0xa30, 0xa46, 0xa02, 0xa21, 0xa00, 0xabb, 0xa1a, 0xad2, 0xa6d, 0xaf9, 0xa30, 0xaa5, 0xa1c, 0xace, 0xa11, 0xa0c, 0xaf4,
                            0xa4f, 0xa7a, 0xa48, 0xac3, 0xac5, 0xa61, 0xafd, 0xa97, 0xa75, 0xa00, 0xab1, 0xaae, 0xa5d, 0xa6b, 0xa6f, 0xad1, 0xa3d,
                            0xa0b, 0xa3f, 0xa4a, 0xa02, 0xa21, 0xa00, 0xac5, 0xab4, 0xa29, 0xa51, 0xaac, 0xaed, 0xaff, 0xa14, 0xaab, 0xaba, 0xa27,
                            0xa36, 0xafd, 0xa57, 0xa4b, 0xadb, 0xa46, 0xa5f, 0xa3e, 0xa6f, 0xa8d, 0xaa1, 0xa2e, 0xa2c, 0xa53, 0xa03, 0xa95, 0xa4a,
                            0xaca, 0xa7f, 0xa78, 0xaf3, 0xa01, // 73-byte signature
                            0xa41, // OP_DATA_65
                            0xa04, 0xaa7, 0xa13, 0xa5b, 0xafe, 0xa82, 0xa4c, 0xa97, 0xaec, 0xac0, 0xa1e, 0xac7, 0xad7, 0xae3, 0xa36, 0xa18, 0xa5c,
                            0xa81, 0xae2, 0xaaa, 0xa2c, 0xa41, 0xaab, 0xa17, 0xa54, 0xa07, 0xac0, 0xa94, 0xa84, 0xace, 0xa96, 0xa94, 0xab4, 0xa49,
                            0xa53, 0xafc, 0xab7, 0xa51, 0xa20, 0xa65, 0xa64, 0xaa9, 0xac2, 0xa4d, 0xad0, 0xa94, 0xad4, 0xa2f, 0xadb, 0xafd, 0xad5,
                            0xaaa, 0xad3, 0xae0, 0xa63, 0xace, 0xa6a, 0xaf4, 0xacf, 0xaaa, 0xaea, 0xa4e, 0xaa1, 0xa4f, 0xabb, // 65-byte pubkey
                        ],
                        sequence: u64::MAX,
                        sig_op_count: 0,
                    }],
                    vec![TransactionOutput {
                        value: 0xf4240,
                        script_public_key: ScriptPublicKey::new(
                            0,
                            scriptvec!(
                                0xa76, // OP_DUP
                                0xaa9, // OP_HASH160
                                0xa14, // OP_DATA_20
                                0xa39, 0xaaa, 0xa3d, 0xa56, 0xa9e, 0xa06, 0xaa1, 0xad7, 0xa92, 0xa6d, 0xac4, 0xabe, 0xa11, 0xa93, 0xac9, 0xa9b, 0xaf2,
                                0xaeb, 0xa9e, 0xae0, 0xa88, // OP_EQUALVERIFY
                                0xaac  // OP_CHECKSIG
                            ),
                        ),
                    }],
                    0,
                    SUBNETWORK_ID_NATIVE,
                    0,
                    vec![],
                ),
            ],
        );

        body_processor.validate_body_in_isolation(&example_block.clone().to_immutable()).unwrap();

        let mut block = example_block.clone();
        let txs = &mut block.transactions;
        txs[1].version += 1;
        assert_match!(body_processor.validate_body_in_isolation(&block.to_immutable()), Err(RuleError::BadMerkleRoot(_, _)));

        let mut block = example_block.clone();
        let txs = &mut block.transactions;
        txs[1].inputs[0].sig_op_count = 255;
        txs[1].inputs[1].sig_op_count = 255;
        block.header.hash_merkle_root = calc_hash_merkle_root(txs.iter());
        assert_match!(body_processor.validate_body_in_isolation(&block.to_immutable()), Err(RuleError::ExceedsMassLimit(_)));

        let mut block = example_block.clone();
        let txs = &mut block.transactions;
        txs.push(txs[1].clone());
        block.header.hash_merkle_root = calc_hash_merkle_root(txs.iter());
        assert_match!(body_processor.validate_body_in_isolation(&block.to_immutable()), Err(RuleError::DuplicateTransactions(_)));

        let mut block = example_block.clone();
        let txs = &mut block.transactions;
        txs[1].subnetwork_id = SUBNETWORK_ID_COINBASE;
        block.header.hash_merkle_root = calc_hash_merkle_root(txs.iter());
        assert_match!(body_processor.validate_body_in_isolation(&block.to_immutable()), Err(RuleError::MultipleCoinbases(_)));

        let mut block = example_block.clone();
        let txs = &mut block.transactions;
        txs[2].inputs[0].previous_outpoint = txs[1].inputs[0].previous_outpoint;
        block.header.hash_merkle_root = calc_hash_merkle_root(txs.iter());
        assert_match!(body_processor.validate_body_in_isolation(&block.to_immutable()), Err(RuleError::DoubleSpendInSameBlock(_)));

        let mut block = example_block.clone();
        let txs = &mut block.transactions;
        txs[0].subnetwork_id = SUBNETWORK_ID_NATIVE;
        block.header.hash_merkle_root = calc_hash_merkle_root(txs.iter());
        assert_match!(body_processor.validate_body_in_isolation(&block.to_immutable()), Err(RuleError::FirstTxNotCoinbase));

        let mut block = example_block.clone();
        let txs = &mut block.transactions;
        txs[1].inputs = vec![];
        block.header.hash_merkle_root = calc_hash_merkle_root(txs.iter());
        assert_match!(
            body_processor.validate_body_in_isolation(&block.to_immutable()),
            Err(RuleError::TxInIsolationValidationFailed(_, _))
        );

        let mut block = example_block;
        let txs = &mut block.transactions;
        txs[3].inputs[0].previous_outpoint = TransactionOutpoint { transaction_id: txs[2].id(), index: 0 };
        block.header.hash_merkle_root = calc_hash_merkle_root(txs.iter());
        assert_match!(body_processor.validate_body_in_isolation(&block.to_immutable()), Err(RuleError::ChainedTransaction(_)));

        consensus.shutdown(wait_handles);
    }

    #[tokio::test]
    async fn merkle_root_missing_parents_known_invalid_test() {
        let config = ConfigBuilder::new(MAINNET_PARAMS).skip_proof_of_work().build();
        let consensus = TestConsensus::new(&config);
        let wait_handles = consensus.init();

        let mut block = consensus.build_block_with_parents_and_transactions(1.into(), vec![config.genesis.hash], vec![]);
        block.transactions[0].version += 1;

        let BlockValidationFutures { block_task, virtual_state_task } =
            consensus.validate_and_insert_block(block.clone().to_immutable());

        assert_match!(block_task.await, Err(RuleError::BadMerkleRoot(_, _)));
        // Assert that both tasks return the same error
        assert_match!(virtual_state_task.await, Err(RuleError::BadMerkleRoot(_, _)));

        // BadMerkleRoot shouldn't mark the block as known invalid
        assert_match!(
            consensus.validate_and_insert_block(block.to_immutable()).virtual_state_task.await,
            Err(RuleError::BadMerkleRoot(_, _))
        );

        let mut block = consensus.build_block_with_parents_and_transactions(1.into(), vec![config.genesis.hash], vec![]);
        block.header.parents_by_level[0][0] = 0.into();

        assert_match!(
            consensus.validate_and_insert_block(block.clone().to_immutable()).virtual_state_task.await,
            Err(RuleError::MissingParents(_))
        );

        // MissingParents shouldn't mark the block as known invalid
        assert_match!(
            consensus.validate_and_insert_block(block.to_immutable()).virtual_state_task.await,
            Err(RuleError::MissingParents(_))
        );

        consensus.shutdown(wait_handles);
    }
}
