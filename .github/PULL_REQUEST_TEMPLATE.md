# Pull Request

## 🎯 Description

Provide a brief description of the changes in this PR.

## 📝 Type of Change

Please select the relevant option(s):

- [ ] 🐛 **Bug fix** (non-breaking change which fixes an issue)
- [ ] 💡 **New feature** (non-breaking change which adds functionality)
- [ ] 💥 **Breaking change** (fix or feature that would cause existing functionality to not work as expected)
- [ ] 📚 **Documentation update** (changes to documentation only)
- [ ] 🎨 **Style/formatting** (changes that do not affect the meaning of the code)
- [ ] ♻️ **Refactoring** (code change that neither fixes a bug nor adds a feature)
- [ ] ⚡ **Performance improvement** (code change that improves performance)
- [ ] 🔒 **Security enhancement** (changes that improve security)
- [ ] 🧪 **Tests** (adding missing tests or correcting existing tests)
- [ ] 🔧 **Chore** (changes to build process, dependencies, etc.)

## 🎨 Category

Which area does this change affect?

- [ ] 🔧 **Core Traits** (Integrator, ForceCalculator, CollisionHandler abstractions)
- [ ] 📐 **Integrators** (VelocityVerlet, Leapfrog, RK4, IAS15 implementations)
- [ ] ⚡ **Force Calculation** (DirectGravity, BarnesHut, FastMultipole algorithms)
- [ ] 🏗️ **Simulation Builder** (configuration, validation, particle management)
- [ ] 🔍 **Collision Detection** (spatial grid, AABB trees, sweep-and-prune)
- [ ] 🧮 **Mathematical Utilities** (vector operations, constants, validation)
- [ ] 🧪 **Testing** (unit tests, integration tests, physics validation, benchmarks)
- [ ] 📚 **Documentation** (API docs, examples, scientific validation guides)
- [ ] 🔒 **Security** (input validation, numerical stability, safe error handling)
- [ ] ⚡ **Performance** (SIMD optimization, parallel computation, memory efficiency)

## 🧪 Testing

Please describe the testing you've performed:

- [ ] Tested in a clean environment
- [ ] Validated physics accuracy with analytical solutions
- [ ] Verified energy conservation over extended simulations
- [ ] Tested performance meets 60fps capability targets
- [ ] Validated cross-platform consistency (native + WASM)
- [ ] Updated and ran existing tests (unit + integration + benchmarks)

**Test Environment:**
- OS:
- Rust Version:
- Target Platform (native/WASM):
- Performance Target (particles/fps achieved):

## 📋 Checklist

### Code Quality

- [ ] My code follows the project's style guidelines
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] My changes generate no new warnings or errors

### Documentation

- [ ] I have made corresponding changes to the documentation
- [ ] I have updated the `HELPFUL_PROMPTS.md` file if applicable
- [ ] I have updated relevant AI instruction files
- [ ] My changes are reflected in code comments where appropriate

### Compatibility

- [ ] My changes maintain backward compatibility
- [ ] I have considered the impact on existing API users
- [ ] I have tested across native and WASM builds
- [ ] My changes maintain scientific accuracy requirements

### Physics Quality

- [ ] My changes maintain deterministic physics behavior
- [ ] I have validated numerical accuracy and stability
- [ ] My code follows Rust scientific computing best practices
- [ ] I have considered performance impact on simulation loops

## 🔗 Related Issues

Link any related issues here:
- Closes #
- Related to #
- Addresses #

## 📸 Screenshots (if applicable)

Add screenshots to help explain your changes, especially for:
- VS Code configuration changes
- New UI elements
- Documentation improvements
- Error messages or fixes

## 🔄 Breaking Changes

If this PR contains breaking changes, please describe:

1. **What breaks:**

2. **Migration path:**

3. **Justification:**

## 🚀 Additional Context

Add any other context about the pull request here:

- Why was this change necessary?
- What alternatives were considered?
- Any performance implications?
- Future considerations or follow-up work?

## 📝 Reviewer Notes

Specific areas you'd like reviewers to focus on:

- [ ] Physics algorithm correctness and numerical accuracy
- [ ] Performance implications and benchmark results
- [ ] Scientific validity and energy conservation
- [ ] API design and ergonomics
- [ ] Documentation clarity and examples
- [ ] Cross-platform compatibility

---

### For Maintainers

**Review Checklist:**
- [ ] Code follows project standards
- [ ] AI instructions are effective and tested
- [ ] Documentation is clear and complete
- [ ] Changes align with project goals
- [ ] No security vulnerabilities introduced
- [ ] Performance impact is acceptable
