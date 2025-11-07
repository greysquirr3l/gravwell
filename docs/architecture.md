# Gravwell Architecture Guide

## Overview

Gravwell is designed as a high-performance, trait-based physics library for
gravity simulation. The architecture emphasizes **zero-cost abstractions**,
**extensibility**, and **dual-mode operation** (game vs. scientific
computing).

## Core Design Principles

1. **Trait-Based Extensibility** - Core functionality defined through traits
2. **Zero-Cost Abstractions** - Compile-time dispatch where possible
3. **SoA Data Layout** - Structure-of-Arrays for SIMD optimization
4. **Dual Performance Modes** - Game (real-time) vs. Science (accuracy)
5. **Minimal Dependencies** - `no_std` core with optional features
6. **Safe by Default** - Leverage Rust's type system for correctness

## High-Level Architecture

```mermaid
classDiagram
    class Simulation {
        +ParticleSet particles
        +Integrator integrator
        +ForceCalculator forces
        +CollisionHandler collisions
        +step(dt: Scalar)
        +energy() Scalar
        +momentum() Vector3
    }

    class ParticleSet {
        +Vec~Vector3~ positions
        +Vec~Vector3~ velocities  
        +Vec~Mass~ masses
        +Vec~Scalar~ radii
        +len() usize
        +add_particle() BodyHandle
        +remove_particle(handle: BodyHandle)
    }

    class BodyHandle {
        +index: usize
        +generation: u32
    }

    Simulation "1" *-- "1" ParticleSet
    ParticleSet "1" *-- "*" BodyHandle
```

## Core Trait System

The architecture centers around three primary traits that define the physics behavior:

```mermaid
classDiagram
    class Integrator {
        <<interface>>
        +step(positions: &mut [Vector3], velocities: &mut [Vector3], forces: &[Vector3], masses: &[Mass], dt: Scalar)
        +name() &str
        +order() u8
        +is_symplectic() bool
    }

    class ForceCalculator {
        <<interface>>
        +calculate_forces(positions: &[Vector3], masses: &[Mass], forces: &mut [Vector3])
        +complexity() Complexity
        +supports_parallel() bool
    }

    class CollisionHandler {
        <<interface>>
        +detect_collisions(positions: &[Vector3], radii: &[Scalar]) Vec~CollisionPair~
        +resolve_collision(pair: &CollisionPair, particles: &mut ParticleSet)
    }

    class Complexity {
        <<enumeration>>
        ON
        ONLogN
        ON2
    }

    ForceCalculator ..> Complexity : uses
```

## Integrator Implementations

Different integrators serve different use cases in the dual-mode architecture:

```mermaid
classDiagram
    class Integrator {
        <<interface>>
        +step()
        +is_symplectic() bool
    }

    class SemiImplicitEuler {
        +step()
        +is_symplectic() bool
        -accelerations: Vec~Vector3~
    }
    
    class VelocityVerlet {
        +step()
        +is_symplectic() bool
        -accelerations: Vec~Vector3~
    }

    class Leapfrog {
        +step()
        +is_symplectic() bool
        -half_velocities: Vec~Vector3~
    }

    class RK4 {
        +step()
        +is_symplectic() bool
        -k1: Vec~Vector3~
        -k2: Vec~Vector3~
        -k3: Vec~Vector3~
        -k4: Vec~Vector3~
    }

    class IAS15 {
        +step()
        +is_symplectic() bool
        +set_tolerance(tol: Scalar)
        -adaptive_step_size: Scalar
        -error_estimate: Scalar
    }

    Integrator <|-- SemiImplicitEuler : "Game Mode: Fast & Stable"
    Integrator <|-- VelocityVerlet : "Balanced: Good for both modes"
    Integrator <|-- Leapfrog : "Science Mode: Symplectic"
    Integrator <|-- RK4 : "Science Mode: High accuracy"
    Integrator <|-- IAS15 : "Science Mode: Adaptive precision"
```

## Force Calculation Algorithms

The force calculation system supports multiple algorithms with different computational complexities:

```mermaid
classDiagram
    class ForceCalculator {
        <<interface>>
        +calculate_forces()
        +complexity() Complexity
    }

    class DirectGravity {
        +calculate_forces()
        +complexity() Complexity
        +set_softening(epsilon: Scalar)
        -softening_parameter: Scalar
    }

    class BarnesHut {
        +calculate_forces()
        +complexity() Complexity
        +set_theta(theta: Scalar)
        -theta: Scalar
        -octree: Octree
    }

    class FastMultipole {
        +calculate_forces()
        +complexity() Complexity
        +set_expansion_order(p: usize)
        -expansion_order: usize
        -multipole_tree: MultipoleTree
    }

    class Octree {
        +insert(particle: Particle)
        +calculate_force_on(particle: Particle) Vector3
        -center: Vector3
        -size: Scalar
        -children: [Option~Box~Octree~~; 8]
        -particles: Vec~Particle~
    }

    ForceCalculator <|-- DirectGravity : "O(N²): Small systems"
    ForceCalculator <|-- BarnesHut : "O(N log N): Medium systems"
    ForceCalculator <|-- FastMultipole : "O(N): Large systems"
    BarnesHut "1" *-- "1" Octree
```

## Data Layout Strategy

Gravwell uses Structure-of-Arrays (SoA) for SIMD-friendly data access:

```mermaid
classDiagram
    class ParticleSet {
        +positions: Vec~Vector3~
        +velocities: Vec~Vector3~
        +masses: Vec~Mass~
        +radii: Vec~Scalar~
        +active: Vec~bool~
        +add_particle() BodyHandle
        +remove_particle(handle: BodyHandle)
        +iter_active() ActiveParticleIter
    }

    class BodyHandle {
        +index: usize
        +generation: u32
        +is_valid(particles: &ParticleSet) bool
    }

    class ActiveParticleIter {
        +next() Option~ParticleRef~
    }

    class ParticleRef {
        +position: &Vector3
        +velocity: &Vector3  
        +mass: &Mass
        +radius: &Scalar
    }

    ParticleSet "1" o-- "*" BodyHandle : manages
    ParticleSet ..> ActiveParticleIter : creates
    ActiveParticleIter ..> ParticleRef : yields

    note for ParticleSet "SoA layout enables SIMD\nvectorization of force calculations"
```

## Performance Optimization Layers

The architecture includes multiple performance optimization layers that can be enabled independently:

```mermaid
classDiagram
    class PerformanceLayer {
        <<interface>>
        +optimize(simulation: &mut Simulation)
    }

    class SIMDOptimizer {
        +optimize()
        +supports_avx() bool
        +supports_avx512() bool
        -instruction_set: InstructionSet
    }

    class ParallelOptimizer {
        +optimize()
        +set_thread_count(count: usize)
        -thread_pool: ThreadPool
    }

    class LODSystem {
        +optimize()
        +set_distance_threshold(threshold: Scalar)
        +update_detail_levels()
        -detail_levels: Vec~DetailLevel~
    }

    class SpatialCuller {
        +optimize()
        +set_view_frustum(frustum: Frustum)
        +cull_particles() Vec~BodyHandle~
        -spatial_index: SpatialIndex
    }

    PerformanceLayer <|-- SIMDOptimizer
    PerformanceLayer <|-- ParallelOptimizer
    PerformanceLayer <|-- LODSystem
    PerformanceLayer <|-- SpatialCuller

    note for LODSystem "Dynamically adjusts simulation\nfidelity based on distance"
    note for SpatialCuller "Removes off-screen particles\nfrom physics calculations"
```

## Error Handling Architecture

Gravwell uses a comprehensive error handling system based on Rust's Result type:

```mermaid
classDiagram
    class GravwellError {
        <<enumeration>>
        InvalidBodyHandle
        InvalidConfiguration
        NumericalInstability
        InsufficientMemory
        GpuError
        ValidationFailed
    }

    class ValidationError {
        +kind: ValidationKind
        +actual_value: Scalar
        +expected_value: Scalar
        +threshold: Scalar
    }

    class ValidationKind {
        <<enumeration>>
        EnergyConservation
        MomentumConservation
        AngularMomentumConservation
        SymplecticProperty
    }

    class Result~T~ {
        +Ok(T)
        +Err(GravwellError)
    }

    GravwellError ..> ValidationError : contains
    ValidationError ..> ValidationKind : uses
    Result ..> GravwellError : error type

    note for Result "All fallible operations return\nResult<T, GravwellError>"
```

## Builder Pattern Implementation

The simulation is constructed using a type-safe builder pattern:

```mermaid
classDiagram
    class SimulationBuilder~I, F~ {
        +new() SimulationBuilder
        +integrator~T~(integrator: T) SimulationBuilder~T, F~
        +forces~T~(calculator: T) SimulationBuilder~I, T~
        +timestep(dt: Scalar) Self
        +gravity_constant(G: Scalar) Self
        +build() Result~Simulation~I, F~~
    }

    class Simulation~I, F~ {
        +step()
        +add_body() Result~BodyHandle~
        +remove_body() Result~()~
        +energy() Scalar
        +momentum() Vector3
    }

    class BuildError {
        <<enumeration>>
        IntegratorNotSet
        ForceCalculatorNotSet  
        InvalidTimestep
        InvalidGravityConstant
    }

    SimulationBuilder --> Simulation : builds
    SimulationBuilder ..> BuildError : can fail with

    note for SimulationBuilder "Type-safe builder ensures\nall required components are set"
```

## Memory Management Strategy

Gravwell employs several memory management strategies for optimal performance:

```mermaid
flowchart TD
    A[Particle Addition Request] --> B{Check Capacity}
    B -->|Sufficient| C[Use Existing Slot]
    B -->|Insufficient| D[Grow Arrays]
    D --> E[Reallocate All Arrays]
    E --> F[Copy Existing Data]
    F --> G[Update Handles]
    C --> H[Insert Particle Data]
    G --> H
    H --> I[Return BodyHandle]

    J[Particle Removal Request] --> K[Mark as Inactive]
    K --> L{Fragmentation Check}
    L -->|High Fragmentation| M[Defragmentation]
    L -->|Low Fragmentation| N[Keep Slot Available]
    M --> O[Compact Arrays]
    O --> P[Update All Handles]
    P --> N
    N --> Q[Return Success]

    style A fill:#e1f5fe
    style I fill:#c8e6c9
    style Q fill:#c8e6c9
    style M fill:#fff3e0
```

## Integration with Game Engines

The architecture provides clean integration points for game engines:

```mermaid
classDiagram
    class GameEngine {
        +update(dt: Scalar)
        +render()
        +handle_input()
    }

    class GravwellPlugin {
        +initialize(engine: &mut GameEngine)
        +update_physics(dt: Scalar)
        +synchronize_transforms()
        -simulation: Simulation
    }

    class Transform {
        +position: Vector3
        +rotation: Quaternion
        +scale: Vector3
    }

    class RenderComponent {
        +mesh: Mesh
        +material: Material
        +visible: bool
    }

    GameEngine "1" o-- "*" GravwellPlugin : manages
    GravwellPlugin "1" *-- "1" Simulation
    GravwellPlugin ..> Transform : updates
    Transform "1" --> "1" RenderComponent : drives

    note for GravwellPlugin "Provides integration layer\nfor popular game engines"
```

## Scientific Validation Framework

Gravwell includes a comprehensive validation system for scientific accuracy:

```mermaid
classDiagram
    class ValidationSuite {
        +run_all_tests() ValidationReport
        +test_energy_conservation() Result~()~
        +test_momentum_conservation() Result~()~
        +test_kepler_orbits() Result~()~
        +compare_with_rebound() Result~()~
    }

    class ValidationReport {
        +passed_tests: usize
        +failed_tests: usize
        +energy_drift: Scalar
        +momentum_drift: Vector3
        +execution_time: Duration
    }

    class KeplerOrbitTest {
        +setup_circular_orbit()
        +setup_elliptical_orbit()
        +verify_period(expected: Scalar, tolerance: Scalar) bool
        +verify_energy_conservation(tolerance: Scalar) bool
    }

    class ReboundComparison {
        +setup_identical_system()
        +run_parallel_simulation()
        +compare_final_states(tolerance: Scalar) bool
    }

    ValidationSuite "1" *-- "1" ValidationReport
    ValidationSuite "1" o-- "*" KeplerOrbitTest
    ValidationSuite "1" o-- "*" ReboundComparison

    note for ValidationSuite "Ensures scientific accuracy\nthrough automated testing"
```

## Module Organization

The crate is organized into logical modules that reflect the architectural layers:

```mermaid
graph TD
    A[lib.rs - Public API] --> B[core/ - Core Abstractions]
    A --> C[integrators/ - Numerical Methods]
    A --> D[forces/ - Force Calculations]
    A --> E[collision/ - Collision Detection]
    A --> F[performance/ - Optimizations]
    A --> G[utils/ - Utilities]

    B --> B1[integrator.rs - Trait Definition]
    B --> B2[forces.rs - Force Calculator Trait]
    B --> B3[particle.rs - Data Structures]
    B --> B4[math.rs - Vector Operations]

    C --> C1[euler.rs - Semi-Implicit Euler]
    C --> C2[verlet.rs - Velocity Verlet]
    C --> C3[leapfrog.rs - Leapfrog]
    C --> C4[rk4.rs - Runge-Kutta 4]
    C --> C5[ias15.rs - IAS15 Adaptive]

    D --> D1[direct.rs - O N squared Direct]
    D --> D2[barnes_hut.rs - O N log N Tree]
    D --> D3[fmm.rs - O N Fast Multipole]

    E --> E1[broad_phase.rs - Spatial Partitioning]
    E --> E2[narrow_phase.rs - Exact Tests]
    E --> E3[response.rs - Collision Response]

    F --> F1[simd.rs - SIMD Optimizations]
    F --> F2[parallel.rs - Multi-threading]
    F --> F3[lod.rs - Level of Detail]
    F --> F4[culling.rs - Spatial Culling]

    G --> G1[constants.rs - Physical Constants]
    G --> G2[validation.rs - Accuracy Testing]
    G --> G3[benchmark.rs - Performance Testing]

    style A fill:#e3f2fd
    style B fill:#f3e5f5
    style C fill:#e8f5e8
    style D fill:#fff3e0
    style E fill:#fce4ec
    style F fill:#f1f8e9
    style G fill:#f5f5f5
```

## Thread Safety and Concurrency

Gravwell is designed with thread safety in mind:

```mermaid
sequenceDiagram
    participant Main as Main Thread
    participant Physics as Physics Thread
    participant Worker1 as Worker Thread 1
    participant Worker2 as Worker Thread 2
    participant GPU as GPU Thread

    Main->>Physics: Start Physics Step
    Physics->>Worker1: Partition 1: Particles 0-499
    Physics->>Worker2: Partition 2: Particles 500-999
    
    par Parallel Force Calculation
        Worker1->>Worker1: Calculate Forces (0-499)
    and
        Worker2->>Worker2: Calculate Forces (500-999)
    end

    Worker1->>Physics: Forces Complete (Partition 1)
    Worker2->>Physics: Forces Complete (Partition 2)
    
    Physics->>Physics: Integrate Positions/Velocities
    
    opt GPU Acceleration Enabled
        Physics->>GPU: Upload Particle Data
        GPU->>GPU: Compute Shader Execution
        GPU->>Physics: Download Results
    end
    
    Physics->>Main: Physics Step Complete
    Main->>Main: Render Frame
```

## Performance Characteristics

The architecture is designed to meet specific performance targets:

| Configuration | Target Performance | Hardware Requirements |
|---------------|-------------------|---------------------|
| Game Mode (Basic) | 1,000 particles @ 30 FPS | Single-core CPU |
| Game Mode (Optimized) | 1,000 particles @ 60 FPS | Multi-core CPU + SIMD |
| Science Mode (Basic) | 10,000 particles @ 1 FPS | Single-core CPU |
| Science Mode (Parallel) | 10,000 particles @ 10 FPS | Multi-core CPU |
| Science Mode (GPU) | 100,000 particles @ 1 FPS | Modern GPU |

## Extension Points

The architecture provides several extension points for customization:

1. **Custom Integrators** - Implement the `Integrator` trait
2. **Custom Force Calculators** - Implement the `ForceCalculator` trait  
3. **Custom Collision Handlers** - Implement the `CollisionHandler` trait
4. **Custom Performance Optimizers** - Implement optimization layers
5. **Custom Validation Tests** - Extend the validation framework

This modular design ensures that Gravwell can be adapted to a wide range of
use cases while maintaining performance and scientific accuracy.
