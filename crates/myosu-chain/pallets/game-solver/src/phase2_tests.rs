#![cfg(test)]

use crate as pallet_game_solver;
use frame_support::{assert_noop, assert_ok, derive_impl};
use sp_core::H256;
use sp_runtime::{
    BuildStorage,
    traits::{BlakeTwo256, IdentityLookup},
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system = 1,
        GameSolver: pallet_game_solver = 2,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Hash = H256;
    type Hashing = BlakeTwo256;
}

impl crate::pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u64;
}

fn new_test_ext() -> sp_io::TestExternalities {
    let mut ext = sp_io::TestExternalities::new(
        frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("frame system storage builds"),
    );
    ext.execute_with(|| System::set_block_number(1));
    ext
}

#[test]
fn register_subnet_tracks_owner_and_next_id() {
    new_test_ext().execute_with(|| {
        assert_ok!(GameSolver::register_subnet(RuntimeOrigin::signed(7)));
        assert_eq!(GameSolver::subnet_owner(0), Some(7));
        assert_eq!(GameSolver::next_subnet_uid(), 1);
    });
}

#[test]
fn register_hotkey_assigns_uid_and_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(GameSolver::register_subnet(RuntimeOrigin::signed(10)));
        assert_ok!(GameSolver::register_hotkey(
            RuntimeOrigin::signed(10),
            0,
            42
        ));

        assert_eq!(GameSolver::hotkey_uid(0, 42), Some(0));
        assert_eq!(GameSolver::owner(42), Some(10));
        assert_eq!(GameSolver::delegate_take(42), 0);
        assert_eq!(GameSolver::next_neuron_uid(0), 1);
    });
}

#[test]
fn register_hotkey_rejects_owner_conflicts() {
    new_test_ext().execute_with(|| {
        assert_ok!(GameSolver::register_subnet(RuntimeOrigin::signed(10)));
        assert_ok!(GameSolver::register_hotkey(
            RuntimeOrigin::signed(10),
            0,
            42
        ));

        assert_ok!(GameSolver::register_subnet(RuntimeOrigin::signed(11)));
        assert_noop!(
            GameSolver::register_hotkey(RuntimeOrigin::signed(11), 1, 42),
            crate::pallet::Error::<Test>::HotkeyOwnedByDifferentColdkey
        );
    });
}

#[test]
fn serving_updates_registered_hotkey_metadata() {
    new_test_ext().execute_with(|| {
        assert_ok!(GameSolver::register_subnet(RuntimeOrigin::signed(10)));
        assert_ok!(GameSolver::register_hotkey(
            RuntimeOrigin::signed(10),
            0,
            42
        ));

        assert_ok!(GameSolver::serve_axon(
            RuntimeOrigin::signed(42),
            0,
            3,
            0x7f00_0001,
            3030,
            4,
            1,
            9,
            8,
            Some(vec![1, 2, 3, 4])
        ));
        assert_ok!(GameSolver::serve_prometheus(
            RuntimeOrigin::signed(42),
            0,
            5,
            0x7f00_0001,
            9090,
            4,
        ));

        let axon = GameSolver::axon_info(0, 42).expect("axon stored");
        assert_eq!(axon.version, 3);
        assert_eq!(axon.port, 3030);
        assert_eq!(axon.protocol, 1);
        assert_eq!(axon.placeholder1, 9);
        assert_eq!(axon.placeholder2, 8);

        let certificate = GameSolver::neuron_certificate(0, 42).expect("certificate stored");
        assert_eq!(certificate.algorithm, 1);
        assert_eq!(certificate.public_key.into_inner(), vec![2, 3, 4]);

        let prometheus = GameSolver::prometheus_info(0, 42).expect("prometheus stored");
        assert_eq!(prometheus.version, 5);
        assert_eq!(prometheus.port, 9090);
    });
}

#[test]
fn serving_rejects_invalid_ip_inputs() {
    new_test_ext().execute_with(|| {
        assert_ok!(GameSolver::register_subnet(RuntimeOrigin::signed(10)));
        assert_ok!(GameSolver::register_hotkey(
            RuntimeOrigin::signed(10),
            0,
            42
        ));

        assert_noop!(
            GameSolver::serve_axon(
                RuntimeOrigin::signed(42),
                0,
                1,
                u32::MAX as u128 + 1,
                3030,
                4,
                0,
                0,
                0,
                None,
            ),
            crate::pallet::Error::<Test>::InvalidIpAddress
        );

        assert_noop!(
            GameSolver::serve_prometheus(RuntimeOrigin::signed(42), 0, 1, 1, 0, 4,),
            crate::pallet::Error::<Test>::InvalidPort
        );
    });
}
