//! Scope management for tracking syntactic nesting and context during parsing.
//!
//! `ScopeManager` tracks brace/bracket depth, current block type, and accumulated
//! content for multi-line directives like `@schema { ... }`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    /// Not inside any block
    None,
    /// Inside `{ ... }` object literal
    Object,
    /// Inside `[ ... ]` list literal
    List,
    /// Inside a directive block like `@schema { ... }`
    DirectiveBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeError {
    UnbalancedExit,
}

/// Tracks syntactic nesting and block context during parsing.
#[derive(Debug)]
pub struct ScopeManager {
    /// Current brace/bracket nesting depth
    nesting_depth: i32,
    /// Current block type
    current_block: BlockType,
    /// Stack of block types for nested structures
    block_stack: Vec<BlockType>,
    /// Accumulated content for multi-line directives
    accumulated_content: String,
}

impl ScopeManager {
    /// Creates a new, empty scope manager
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nesting_depth: 0,
            current_block: BlockType::None,
            block_stack: Vec::new(),
            accumulated_content: String::new(),
        }
    }

    /// Returns the current nesting depth
    #[must_use]
    pub const fn nesting_depth(&self) -> i32 {
        self.nesting_depth
    }

    /// Returns the current block type
    #[must_use]
    pub const fn current_block(&self) -> BlockType {
        self.current_block
    }

    /// Returns true if we're currently inside any nested structure
    #[must_use]
    pub const fn in_nested_context(&self) -> bool {
        self.nesting_depth > 0
    }

    /// Enters a new block (opening brace or bracket)
    pub fn enter_block(&mut self, block_type: BlockType) {
        self.block_stack.push(self.current_block);
        self.current_block = block_type;
        self.nesting_depth += 1;
    }

    /// Exits a block (closing brace or bracket)
    ///
    /// Returns `Ok(())` if the block nesting is balanced, or `Err(())` if
    /// we're trying to exit without having entered.
    ///
    /// # Errors
    ///
    /// Returns `Err(ScopeError::UnbalancedExit)` when an exit is attempted
    /// while the nesting depth is already zero.
    pub fn exit_block(&mut self) -> Result<(), ScopeError> {
        if self.nesting_depth <= 0 {
            return Err(ScopeError::UnbalancedExit);
        }
        self.nesting_depth -= 1;
        self.current_block = self.block_stack.pop().unwrap_or(BlockType::None);
        Ok(())
    }

    /// Returns the accumulated content for multi-line directives
    #[must_use]
    pub fn accumulated_content(&self) -> &str {
        &self.accumulated_content
    }

    /// Appends text to the accumulated content
    pub fn accumulate(&mut self, text: &str) {
        if !self.accumulated_content.is_empty() {
            self.accumulated_content.push(' ');
        }
        self.accumulated_content.push_str(text);
    }

    /// Clears the accumulated content
    pub fn clear_accumulated(&mut self) {
        self.accumulated_content.clear();
    }

    /// Returns true if a multi-line block has been completed (balanced braces)
    #[must_use]
    pub const fn block_is_complete(&self) -> bool {
        self.nesting_depth == 0 && !self.accumulated_content.is_empty()
    }

    /// Resets the scope manager to the initial state
    pub fn reset(&mut self) {
        self.nesting_depth = 0;
        self.current_block = BlockType::None;
        self.block_stack.clear();
        self.accumulated_content.clear();
    }
}

impl Default for ScopeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_nesting() {
        let mut scope = ScopeManager::new();
        assert_eq!(scope.nesting_depth(), 0);
        assert!(!scope.in_nested_context());

        scope.enter_block(BlockType::Object);
        assert_eq!(scope.nesting_depth(), 1);
        assert!(scope.in_nested_context());
        assert_eq!(scope.current_block(), BlockType::Object);

        scope.exit_block().unwrap();
        assert_eq!(scope.nesting_depth(), 0);
        assert!(!scope.in_nested_context());
    }

    #[test]
    fn test_nested_objects() {
        let mut scope = ScopeManager::new();
        scope.enter_block(BlockType::Object);
        scope.enter_block(BlockType::Object);
        assert_eq!(scope.nesting_depth(), 2);

        scope.exit_block().unwrap();
        assert_eq!(scope.current_block(), BlockType::Object);

        scope.exit_block().unwrap();
        assert_eq!(scope.current_block(), BlockType::None);
    }

    #[test]
    fn test_exit_without_enter() {
        let mut scope = ScopeManager::new();
        assert!(scope.exit_block().is_err());
    }

    #[test]
    fn test_accumulation() {
        let mut scope = ScopeManager::new();
        assert_eq!(scope.accumulated_content(), "");

        scope.accumulate("first");
        assert_eq!(scope.accumulated_content(), "first");

        scope.accumulate("second");
        assert_eq!(scope.accumulated_content(), "first second");

        scope.clear_accumulated();
        assert_eq!(scope.accumulated_content(), "");
    }

    #[test]
    fn test_block_complete() {
        let mut scope = ScopeManager::new();
        assert!(!scope.block_is_complete());

        scope.accumulate("content");
        assert!(scope.block_is_complete());

        scope.clear_accumulated();
        assert!(!scope.block_is_complete());
    }
}
