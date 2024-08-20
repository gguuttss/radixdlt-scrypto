use radix_engine::system::system_db_reader::*;
use radix_engine::transaction::*;
use radix_engine::updates::*;
use radix_engine::vm::NoExtension;
use radix_substate_store_impls::memory_db::*;
use radix_substate_store_impls::substate_database_overlay::*;
use radix_substate_store_interface::db_key_mapper::*;
use radix_substate_store_interface::interface::*;
use radix_transaction_scenarios::executor::*;
use radix_transactions::builder::*;
use scrypto::prelude::*;
use scrypto_test::ledger_simulator::*;
use scrypto_test::prelude::DefaultWasmEngine;

#[test]
fn substates_written_to_root_database_can_be_read() {
    // Arrange
    let mut root = InMemorySubstateDatabase::standard();
    root.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Delta {
                        substate_updates: indexmap! {
                            DbSortKey(b"some-sort-key".to_vec()) => DatabaseUpdate::Set(
                                b"some-substate-value".to_vec()
                            )
                        }
                    }
                }
            }
        },
    });

    let db = SubstateDatabaseOverlay::new_unmergeable(&root);

    // Act
    let substate = db.get_substate(
        &DbPartitionKey {
            node_key: b"some-node".to_vec(),
            partition_num: 0,
        },
        &DbSortKey(b"some-sort-key".to_vec()),
    );

    // Assert
    assert_eq!(substate, Some(b"some-substate-value".to_vec()))
}

#[test]
fn substates_written_to_overlay_can_be_read_later() {
    // Arrange
    let root = InMemorySubstateDatabase::standard();
    let mut db = SubstateDatabaseOverlay::new_unmergeable(&root);

    db.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Delta {
                        substate_updates: indexmap! {
                            DbSortKey(b"some-sort-key".to_vec()) => DatabaseUpdate::Set(
                                b"some-substate-value".to_vec()
                            )
                        }
                    }
                }
            }
        },
    });

    // Act
    let substate = db.get_substate(
        &DbPartitionKey {
            node_key: b"some-node".to_vec(),
            partition_num: 0,
        },
        &DbSortKey(b"some-sort-key".to_vec()),
    );

    // Assert
    assert_eq!(substate, Some(b"some-substate-value".to_vec()))
}

#[test]
fn substate_deletes_to_overlay_prevent_substate_from_being_read() {
    // Arrange
    let mut root = InMemorySubstateDatabase::standard();
    root.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Delta {
                        substate_updates: indexmap! {
                            DbSortKey(b"some-sort-key".to_vec()) => DatabaseUpdate::Set(
                                b"some-substate-value".to_vec()
                            )
                        }
                    }
                }
            }
        },
    });

    let mut db = SubstateDatabaseOverlay::new_unmergeable(&root);
    db.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Delta {
                        substate_updates: indexmap! {
                            DbSortKey(b"some-sort-key".to_vec()) => DatabaseUpdate::Delete
                        }
                    }
                }
            }
        },
    });

    // Act
    let substate = db.get_substate(
        &DbPartitionKey {
            node_key: b"some-node".to_vec(),
            partition_num: 0,
        },
        &DbSortKey(b"some-sort-key".to_vec()),
    );

    // Assert
    assert_eq!(substate, None)
}

#[test]
fn partition_deletes_to_overlay_prevent_substate_from_being_read() {
    // Arrange
    let mut root = InMemorySubstateDatabase::standard();
    root.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Delta {
                        substate_updates: indexmap! {
                            DbSortKey(b"some-sort-key".to_vec()) => DatabaseUpdate::Set(
                                b"some-substate-value".to_vec()
                            )
                        }
                    }
                }
            }
        },
    });

    let mut db = SubstateDatabaseOverlay::new_unmergeable(&root);
    db.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Reset {
                        new_substate_values: indexmap!{}
                    }
                }
            }
        },
    });

    // Act
    let substate = db.get_substate(
        &DbPartitionKey {
            node_key: b"some-node".to_vec(),
            partition_num: 0,
        },
        &DbSortKey(b"some-sort-key".to_vec()),
    );

    // Assert
    assert_eq!(substate, None)
}

#[test]
fn partition_resets_to_overlay_return_new_substate_data() {
    // Arrange
    let mut root = InMemorySubstateDatabase::standard();
    root.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Delta {
                        substate_updates: indexmap! {
                            DbSortKey(b"some-sort-key".to_vec()) => DatabaseUpdate::Set(
                                b"some-substate-value".to_vec()
                            )
                        }
                    }
                }
            }
        },
    });

    let mut db = SubstateDatabaseOverlay::new_unmergeable(&root);
    db.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Reset {
                        new_substate_values: indexmap!{
                            DbSortKey(b"some-sort-key".to_vec()) => b"some-other-value".to_vec()
                        }
                    }
                }
            }
        },
    });

    // Act
    let substate = db.get_substate(
        &DbPartitionKey {
            node_key: b"some-node".to_vec(),
            partition_num: 0,
        },
        &DbSortKey(b"some-sort-key".to_vec()),
    );

    // Assert
    assert_eq!(substate, Some(b"some-other-value".to_vec()))
}

#[test]
fn partition_resets_are_not_combined() {
    // Arrange
    let mut root = InMemorySubstateDatabase::standard();
    root.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Delta {
                        substate_updates: indexmap! {
                            DbSortKey(b"some-sort-key".to_vec()) => DatabaseUpdate::Set(
                                b"some-substate-value".to_vec()
                            )
                        }
                    }
                }
            }
        },
    });

    let mut db = SubstateDatabaseOverlay::new_unmergeable(&root);
    db.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Reset {
                        new_substate_values: indexmap!{
                            DbSortKey(b"some-sort-key".to_vec()) => b"some-other-value".to_vec()
                        }
                    }
                }
            }
        },
    });
    db.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Reset {
                        new_substate_values: indexmap!{}
                    }
                }
            }
        },
    });

    // Act
    let substate = db.get_substate(
        &DbPartitionKey {
            node_key: b"some-node".to_vec(),
            partition_num: 0,
        },
        &DbSortKey(b"some-sort-key".to_vec()),
    );

    // Assert
    assert_eq!(substate, None)
}

#[test]
fn from_sort_key_in_list_entries_from_works_when_the_overlay_is_in_reset_mode() {
    // Arrange
    let root = InMemorySubstateDatabase::standard();
    let mut db = SubstateDatabaseOverlay::new_unmergeable(&root);

    db.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Reset {
                        new_substate_values: indexmap!{
                            DbSortKey([0].to_vec()) => b"0".to_vec(),
                            DbSortKey([1].to_vec()) => b"1".to_vec(),
                            DbSortKey([2].to_vec()) => b"2".to_vec()
                        }
                    }
                }
            }
        },
    });

    // Act
    let mut substates = db.list_entries_from(
        &DbPartitionKey {
            node_key: b"some-node".to_vec(),
            partition_num: 0,
        },
        Some(&DbSortKey([1].to_vec())),
    );

    // Assert
    let substate1 = substates.next().expect("We must get the first substate");
    let substate2 = substates.next().expect("We must get the first substate");
    assert_eq!(
        substates.next(),
        None,
        "Another substate is available after the two substates"
    );

    assert_eq!(substate1, (DbSortKey([1].to_vec()), b"1".to_vec()));
    assert_eq!(substate2, (DbSortKey([2].to_vec()), b"2".to_vec()));
}

#[test]
fn from_sort_key_in_list_entries_from_works_when_the_overlay_is_in_delta_mode() {
    // Arrange
    let root = InMemorySubstateDatabase::standard();
    let mut db = SubstateDatabaseOverlay::new_unmergeable(&root);

    db.commit(&DatabaseUpdates {
        node_updates: indexmap! {
            b"some-node".to_vec() => NodeDatabaseUpdates {
                partition_updates: indexmap! {
                    0 => PartitionDatabaseUpdates::Delta {
                        substate_updates: indexmap!{
                            DbSortKey([0].to_vec()) => DatabaseUpdate::Set(b"0".to_vec()),
                            DbSortKey([1].to_vec()) => DatabaseUpdate::Set(b"1".to_vec()),
                            DbSortKey([2].to_vec()) => DatabaseUpdate::Set(b"2".to_vec())
                        }
                    }
                }
            }
        },
    });

    // Act
    let mut substates = db.list_entries_from(
        &DbPartitionKey {
            node_key: b"some-node".to_vec(),
            partition_num: 0,
        },
        Some(&DbSortKey([1].to_vec())),
    );

    // Assert
    let substate1 = substates.next().expect("We must get the first substate");
    let substate2 = substates.next().expect("We must get the first substate");
    assert_eq!(
        substates.next(),
        None,
        "Another substate is available after the two substates"
    );

    assert_eq!(substate1, (DbSortKey([1].to_vec()), b"1".to_vec()));
    assert_eq!(substate2, (DbSortKey([2].to_vec()), b"2".to_vec()));
}

#[test]
fn substates_written_on_a_staging_database_from_transactions_can_be_read_later() {
    // Arrange
    let root_database = InMemorySubstateDatabase::standard();
    let database = SubstateDatabaseOverlay::new_unmergeable(&root_database);
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_database(database)
        .without_kernel_trace()
        .build();

    let (public_key1, _, account1) = ledger.new_account(false);
    let (public_key2, _, account2) = ledger.new_account(false);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account1, XRD, dec!(100))
            .deposit_batch(account2)
            .build(),
        [public_key1, public_key2]
            .map(|pk| NonFungibleGlobalId::from_public_key(&pk))
            .to_vec(),
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn transaction_receipts_from_scenarios_are_identical_between_staging_and_non_staging_database() {
    run_scenarios(|(_, non_staging_receipt), (_, staging_receipt)| {
        assert_eq!(non_staging_receipt, staging_receipt)
    })
}

#[test]
#[allow(clippy::redundant_closure_call)]
fn database_hashes_are_identical_between_staging_and_non_staging_database_at_each_scenario_step() {
    run_scenarios(|(non_staging_database, _), (staging_database, _)| {
        let non_staging_database_hash = create_database_contents_hash(non_staging_database);
        let staging_database_hash = create_database_contents_hash(staging_database);

        assert_eq!(non_staging_database_hash, staging_database_hash)
    })
}

fn create_database_contents_hash<D: SubstateDatabase + ListableSubstateDatabase>(
    database: &D,
) -> Hash {
    let mut accumulator_hash = Hash([0; 32]);
    let reader = SystemDatabaseReader::new(database);
    for (node_id, partition_number) in reader.partitions_iter() {
        let db_node_key = SpreadPrefixKeyMapper::to_db_node_key(&node_id);
        let db_partition_key = DbPartitionKey {
            node_key: db_node_key,
            partition_num: partition_number.0,
        };

        for (substate_key, substate_value) in
            SubstateDatabase::list_entries(database, &db_partition_key)
        {
            let entry_hash = hash(
                scrypto_encode(&(node_id, partition_number, substate_key, substate_value)).unwrap(),
            );
            let mut data = accumulator_hash.to_vec();
            data.extend(entry_hash.to_vec());
            accumulator_hash = hash(data);
        }
    }
    accumulator_hash
}

/// Runs the scenarios on an [`InMemorySubstateDatabase`] and a [`SingleSubstateDatabaseOverlay`] wrapping
/// an [`InMemorySubstateDatabase`]. The passed check function is executed after the execution of
/// each scenario.
fn run_scenarios(
    check_callback: impl Fn(
        (&InMemorySubstateDatabase, &TransactionReceipt),
        (
            &UnmergeableSubstateDatabaseOverlay<'_, InMemorySubstateDatabase>,
            &TransactionReceipt,
        ),
    ),
) {
    let overlay_root = InMemorySubstateDatabase::standard();
    let overlay = SubstateDatabaseOverlay::new_unmergeable(&overlay_root);
    let ledger_with_overlay = Rc::new(RefCell::new(
        LedgerSimulatorBuilder::new()
            .with_custom_database(overlay)
            .with_protocol_version(ProtocolVersion::Babylon)
            .without_kernel_trace()
            .build(),
    ));
    let network_definition = NetworkDefinition::simulator();

    struct ProtocolUpdateHooks<'a> {
        ledger_with_overlay: Rc<RefCell<LedgerSimulator<NoExtension, SubstateDatabaseOverlay<&'a InMemorySubstateDatabase, InMemorySubstateDatabase>>>>,
    }

    impl<'a> ProtocolUpdateExecutionHooks for ProtocolUpdateHooks<'a> {
        const IS_ENABLED: bool = true;
        type WasmEngine = DefaultWasmEngine;
        type NativeVmExtension = NoExtension;
        
        fn get_vm_extension(&mut self) -> NoExtension {
            NoExtension
        }
        
        fn on_transaction_executed(
            &mut self,
            _protocol_version: ProtocolVersion,
            _batch_group_index: usize,
            _batch_group_name: &str,
            _batch_index: usize,
            _transaction_num: usize,
            _transaction: &ProtocolUpdateTransactionDetails,
            receipt: &TransactionReceipt,
            _resultant_store: &dyn SubstateDatabase,
        ) {
            // We copy the protocol updates onto the ledger_with_overlay
            let database_updates = receipt.expect_commit_success().state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
            self.ledger_with_overlay.borrow_mut().substate_db_mut().commit(&database_updates);
        }
    }

    DefaultTransactionScenarioExecutor::new(
        InMemorySubstateDatabase::standard(),
        &network_definition,
    )
    .on_transaction_executed(|_, transaction, receipt, db| {
        // Execute the same transaction on the ledger simulator.
        let receipt_from_overlay = ledger_with_overlay
            .borrow_mut()
            .execute_notarized_transaction(&transaction.raw_transaction);

        // Check that everything matches.
        check_callback(
            (db, &receipt),
            (
                ledger_with_overlay.borrow().substate_db(),
                &receipt_from_overlay,
            ),
        );
    })
    .execute_protocol_updates_and_scenarios(
        ProtocolBuilder::for_network(&network_definition).bootstrap_then_until(ProtocolVersion::LATEST),
        ScenarioTrigger::AtStartOfEveryProtocolVersion,
        ScenarioFilter::AllScenariosFirstValidAtProtocolVersion,
        &mut ProtocolUpdateHooks {
            ledger_with_overlay: ledger_with_overlay.clone(),
        },
    )
    .expect("Must succeed!");
}
