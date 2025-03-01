# Contributing Guidelines

Thank you for your interest in contributing to the Term-kit! We welcome contributions from everyone. Here's how you can help:

## Table of Contents
- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Reporting Issues](#reporting-issues)
- [Feature Requests](#feature-requests)
- [Pull Request Process](#pull-request-process)
- [Style Guide](#style-guide)

## Code of Conduct
Please read our [Code of Conduct](CODE_OF_CONDUCT.md) before participating. We enforce a respectful and inclusive environment.

## Getting Started
1. **Fork the Repository**  
   Click the "Fork" button at the top-right of this repository.

2. **Clone Your Fork**  
   ```bash
   git clone https://github.com/WilsonHuang080705/term-kit.git
   cd term-kit
   ```

3. **Set Upstream**  
   ```bash
   git remote add upstream https://github.com/WilsonHuang080705/term-kit.git
   ```

4. **Install Dependencies**  
   ```bash
   cargo r  # or other package manager commands
   ```

## Development Workflow
1. Create a new branch:  
   ```bash
   git checkout -b feat/your-feature-name   # or fix/your-bugfix-name
   ```

2. Make your changes and test them thoroughly.

3. Commit your changes:  
   ```bash
   git commit -m "feat: add new feature"   # Follow [Conventional Commits](https://www.conventionalcommits.org)
   ```

## Reporting Issues
- Check existing issues before creating new ones
- Use the issue template (if available)
- Include:
  - OS/environment details
  - Steps to reproduce
  - Expected vs actual behavior
  - Screenshots (if applicable)

## Feature Requests
- Describe the problem your feature solves
- Suggest alternative solutions
- Include mockups or examples (if possible)

## Pull Request Process
1. Keep PRs focused on a single feature/bugfix
2. Update documentation if needed
3. Ensure tests pass (if applicable)
4. Reference related issues using `#issue-number`
5. Maintainers will review your PR within 3-5 business days

## Style Guide
### Code
- Follow existing patterns in the codebase
- Use 2-space indentation (unless language-specific standards differ)
- Add comments for complex logic

### Commit Messages
- Use [Conventional Commits](https://www.conventionalcommits.org) format:
  ```
  type(scope): description
  ```
  - Common types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

### Documentation
- Keep Markdown files at 80-character line width
- Use semantic line breaks

---

Need help? Contact [maintainer@email.com] or join our (Discussions)[https://t.me/MatrixHuangShare]
