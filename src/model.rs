use anyhow::{Context, anyhow};
use hoomd_geometry::{Volume, shape::{EightEight, HyperbolicConvexPolytope}};
use hoomd_interaction::{
    MaximumInteractionRange, PairwiseCutoff,
    pairwise::{HardShape, Isotropic},
    univariate::{Expanded, LennardJones, OverlapPenalty},
};
use hoomd_manifold::{Hyperbolic, HyperbolicDisk, Minkowski};
use hoomd_mc::{BodyDistribution, Count, QuickCompress, QuickInsert, Rotate, Sweep, Translate, Trial, Tune, UniformIn};
use hoomd_microstate::{Body, Microstate, SiteKey, boundary::Periodic, property::OrientedHyperbolicPoint};
use hoomd_simulation::{Simulation, macrostate::Isothermal};
use hoomd_spatial::AllPairs;
use hoomd_vector::Angle;
use log::debug;
use rand::{Rng, distr::Distribution};
use serde::{Deserialize, Serialize};

use crate::state_point;

use super::StatePoint;

const NUM_STEPS: u64 = 10_000;

type PositionVector = Hyperbolic<3>;
type Orientation = Angle;
type SiteProperties = OrientedHyperbolicPoint<3, Angle>;
type BodyProperties = OrientedHyperbolicPoint<3, Angle>;
type Boundary = Periodic<EightEight>;

#[derive(Serialize, Deserialize)]
pub enum Phase {
    Initialize,
    Crunch,
    Equilibrate,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct UniformHyperbolic<S> {
    template_sites: Vec<S>,
}

impl BodyDistribution<Body<OrientedHyperbolicPoint<3, Angle>>>
    for UniformHyperbolic<OrientedHyperbolicPoint<3, Angle>>
{
    #[inline]
    fn sample<R: Rng + ?Sized>(
        &self,
        _index: usize,
        rng: &mut R,
    ) -> Body<
        OrientedHyperbolicPoint<3, Angle>,
        OrientedHyperbolicPoint<3, Angle>,
    > {
        let initial_spacing = 1.4;
        let sample_disk = HyperbolicDisk {
            disk_radius: initial_spacing.try_into().expect("positive number"),
            point: Hyperbolic::<3>::from_minkowski_coordinates(
                Minkowski::from([
                    0.00001,
                    0.00001,
                    f64::sqrt(2.0 * (0.00001_f64).powi(2) + RHO.powi(2)),
                ]),
            ),
        };
        let new_point: Hyperbolic<3> = Hyperbolic::from_minkowski_coordinates(
            *sample_disk.sample(rng).point()
        );
        //let new_angle: Angle = rng.random();
        let body_properties = OrientedHyperbolicPoint {
            position: new_point,
            orientation: Angle::default(), //new_angle,
        };
        let site_properties = OrientedHyperbolicPoint {
            position: Hyperbolic::<3>::default(),
            orientation: Angle::default(), //new_angle,
        };
        Body {
            properties: body_properties,
            sites: vec![site_properties],
        }
    }
}
/// The Lennard-Jones simulation model.
#[derive(Serialize, Deserialize)]
pub struct HyperbolicSquaresModel {
    pub microstate: Microstate<BodyProperties, SiteProperties, AllPairs<SiteKey>, Boundary>,
    pub translate_sweep: Sweep<Translate<OrientedHyperbolicPoint<3,Angle>>>,
    pub rotate_sweep: Sweep<Rotate<Orientation>>,
    pub hamiltonian: PairwiseCutoff<HardShape<HyperbolicConvexPolytope<3>>>,
    pub quick_insert: QuickInsert<UniformHyperbolic<SiteProperties>>,
    pub insert_hamiltonian: PairwiseCutoff<Isotropic<LennardJones>>,
    pub macrostate: Isothermal,
    pub translate_count: Count,
    pub phase: Phase,
    pub relax_step: u64,
    pub end_size: f64,
}

impl Simulation for HyperbolicSquaresModel {
    #[inline]
    fn advance(&mut self) -> anyhow::Result<()> {
        match self.phase {
            Phase::Initialize => self.initialize().context("failed to initialize")?,
            Phase::Crunch => self.crunch(),
            Phase::Equilibrate => self.equilibrate(),
        }

        self.microstate.increment_step();

        Ok(())
    }

    #[inline]
    fn step(&self) -> u64 {
        self.microstate.step()
    }
}

impl HyperbolicSquaresModel {
    pub fn new(state_point: StatePoint) -> anyhow::Result<Self> {
        let maximum_distance = state_point.final_size * 0.001;
        let maximum_rotation = 0.01;
        let macrostate = Isothermal { temperature: 1.0 };

        let end_square =
            HyperbolicConvexPolytope::<3>::regular(4, state_point.final_size);
        let hamiltonian = PairwiseCutoff(HardShape(end_square.clone()));
        let boundary = Periodic::new(0.6, EightEight {})?;
        //let allpairs = AllPairs
        let microstate = Microstate::builder()
            //.spatial_data(allpairs)
            .seed(state_point.replicate)
            .boundary(boundary)
            .try_build()?;

        let hyp_translate =
            Translate::with_maximum_distance(maximum_distance.try_into()?);
        let translate_sweep = Sweep(hyp_translate);

        let rotate =
            Rotate::with_maximum_rotation(maximum_rotation.try_into()?);
        let rotate_sweep = Sweep(rotate);

        let distribution = UniformHyperbolic {
            template_sites: vec![OrientedHyperbolicPoint::<3, Angle>::default()],
        };
        let quick_insert = QuickInsert::new(distribution, state_point.n);

        let lj: LennardJones = LennardJones {
            epsilon: 10.0,
            sigma: state_point.final_size * 0.2,
        };

        let insert_hamiltonian = PairwiseCutoff(Isotropic {
            interaction: lj,
            r_cut: 1.0,
        });

        Ok(HyperbolicSquaresModel {
            microstate,
            hamiltonian,
            translate_sweep,
            rotate_sweep,
            quick_insert,
            insert_hamiltonian,
            macrostate,
            phase: Phase::Initialize,
            translate_count: Count::default(),
            relax_step: 0,
            end_size: state_point.final_size,
        })
    }

    fn initialize(&mut self) -> anyhow::Result<()> {
        self.quick_insert
            .apply(&mut self.microstate, &self.insert_hamiltonian);

        self.translate_sweep.apply(
            &mut self.microstate,
            &self.insert_hamiltonian,
            &Isothermal { temperature: 1.0 },
        );

        self.rotate_sweep.apply(
            &mut self.microstate,
            &self.insert_hamiltonian,
            &Isothermal { temperature: 1.0 },
        );

        if self.quick_insert.is_complete() {
            self.phase = Phase::Crunch;
            println!(
                "Initialization complete at step {}.",
                self.microstate.step()
            );
        }

        if self.step() >= 10_000 {
            let n = self.microstate.bodies().len();
            let target = self.quick_insert.target();
            let step = self.microstate.step();
            return Err(anyhow!(
                "{n} of {target} bodies inserted after {step} steps"
            ));
        }

        Ok(())
    }

    fn crunch(&mut self) {
        let step = self.microstate.step();
        let radius = self.end_size * (0.)
            * ((step as f64) / (NUM_STEPS as f64))
            + 0.1 * self.end_size;

        let crunch_square =
            HyperbolicConvexPolytope::<3>::regular(4, radius);
        let crunch_hamiltonian =
            PairwiseCutoff(HardShape(crunch_square.clone()));

        self.translate_sweep.apply(
            &mut self.microstate,
            &crunch_hamiltonian,
            &Isothermal { temperature: 1.0 },
        );

        self.rotate_sweep.apply(
            &mut self.microstate,
            &crunch_hamiltonian,
            &Isothermal { temperature: 1.0 },
        );

        if step > NUM_STEPS {
            self.phase = Phase::Equilibrate;
        }
    }

    fn equilibrate(&mut self) {
        self.translate_sweep.apply(
            &mut self.microstate,
            &self.hamiltonian,
            &self.macrostate,
        );
        self.rotate_sweep.apply(
            &mut self.microstate,
            &self.hamiltonian,
            &self.macrostate,
        );
    }

    pub fn clear_move_counts(&mut self) {
        self.translate_count = Count::default()
    }
}
