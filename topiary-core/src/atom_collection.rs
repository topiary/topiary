use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    mem,
    ops::Deref,
};

use topiary_tree_sitter_facade::Node;

use crate::{
    Atom, Capitalisation, FormatterError, FormatterResult, ScopeCondition, ScopeInformation,
    tree_sitter::NodeExt,
};

/// A struct that holds sets of node IDs that have line breaks before or after them.
///
/// This struct is used by the `detect_line_breaks` function to return the node IDs that
/// have at least `minimum_line_breaks` number of line breaks before or after them in the
/// flattened vector of nodes.
struct NodesWithLinebreaks {
    /// A set of node IDs that have line breaks before them.
    before: HashSet<usize>,
    /// A set of node IDs that have line breaks after them.
    after: HashSet<usize>,
}

/// Contains Topiary's internal representation parsed document.
#[derive(Debug)]
pub struct AtomCollection {
    /// A flat list of all Atoms. This is is updated by some formatting
    /// directives, but most require some more complexity.
    atoms: Vec<Atom>,
    /// Whenever a formatting directive instructs tree-sitter to prepend
    /// something to a node, a new Atom is added to this HashMap.
    /// The key of the hashmap is the identifier of the node.
    prepend: HashMap<usize, Vec<Atom>>,
    /// Whenever a formatting directive instructs tree-sitter to append
    /// something to a node, a new Atom is added to this HashMap.
    /// The key of the hashmap is the identifier of the node.
    append: HashMap<usize, Vec<Atom>>,
    /// A query file can define custom leaf nodes (nodes that Topiary should not
    /// touch during formatting). When such a node is encountered, its id is stored in
    /// this HashSet.
    specified_leaf_nodes: HashSet<usize>,
    /// If a node is a leaf, or if it is explicitly marked as such by the
    /// formatting directives, it is added to this HashMap as the key. The value
    /// of the Map contains all parent nodes.
    parent_leaf_nodes: HashMap<usize, usize>,
    /// Topiary has some formatting directives that only apply if a node spans
    /// multiple lines. During initial collection all such nodes are added to this
    /// HashSet for easy checking if a node spans multiple lines.
    multi_line_nodes: HashSet<usize>,
    /// During initial Atom collection, any node that has a blank lines above
    /// the node is added to this HashSet.
    blank_lines_before: HashSet<usize>,
    /// During initial Atom collection, any node that has a linebreak directly
    /// before it is added to this HashSet.
    line_break_before: HashSet<usize>,
    /// During initial Atom collection, any node that has a linebreak directly
    /// after it is added to this HashSet.
    line_break_after: HashSet<usize>,
    /// Used to generate unique IDs
    counter: usize,
    /// (`@injection.content`, `@injection.language`) values
    injection_content: HashSet<(usize, String)>,
    injection_language: Option<String>,
}

impl AtomCollection {
    /// Returns a basic AtomCollection with the supplied atoms. Only used for
    /// testing. Normally you should use `AtomCollection::collect_leaves`
    /// instead.
    #[cfg(test)]
    pub fn new(atoms: Vec<Atom>) -> Self {
        Self {
            atoms,
            prepend: HashMap::new(),
            append: HashMap::new(),
            specified_leaf_nodes: HashSet::new(),
            parent_leaf_nodes: HashMap::new(),
            multi_line_nodes: HashSet::new(),
            blank_lines_before: HashSet::new(),
            line_break_before: HashSet::new(),
            line_break_after: HashSet::new(),
            counter: 0,
            injection_content: HashSet::new(),
            injection_language: None,
        }
    }

    /// Use this to create an initial `AtomCollection`
    pub fn collect_leaves(
        root: &Node,
        source: &[u8],
        specified_leaf_nodes: HashSet<usize>,
    ) -> FormatterResult<Self> {
        // Flatten the tree, from the root node, in a depth-first traversal
        let dfs_nodes = dfs_flatten(root);

        // Detect user specified line breaks
        let multi_line_nodes = detect_multi_line_nodes(&dfs_nodes);
        let blank_line_nodes = detect_line_breaks(&dfs_nodes, 2);
        let line_break_nodes = detect_line_breaks(&dfs_nodes, 1);

        let mut atoms = Self {
            atoms: Vec::new(),
            prepend: HashMap::new(),
            append: HashMap::new(),
            specified_leaf_nodes,
            parent_leaf_nodes: HashMap::new(),
            multi_line_nodes,
            blank_lines_before: blank_line_nodes.before,
            line_break_before: line_break_nodes.before,
            line_break_after: line_break_nodes.after,
            counter: 0,
            injection_content: HashSet::new(),
            injection_language: None,
        };

        atoms.collect_leaves_inner(root, source, &Vec::new(), 0)?;

        Ok(atoms)
    }

    // wrap inside a conditional atom if #single/multi_line_scope_only! is set
    fn wrap(&mut self, atom: Atom, predicates: &QueryPredicates) -> Atom {
        if let Some(scope_id) = &predicates.single_line_scope_only {
            let id = self.next_id();
            Atom::ScopedConditional {
                id,
                scope_id: scope_id.to_string(),
                condition: ScopeCondition::SingleLineOnly,
                atom: Box::new(atom),
            }
        } else if let Some(scope_id) = &predicates.multi_line_scope_only {
            let id = self.next_id();
            Atom::ScopedConditional {
                id,
                scope_id: scope_id.to_string(),
                condition: ScopeCondition::MultiLineOnly,
                atom: Box::new(atom),
            }
        } else {
            atom
        }
    }

    /// Resolves a capture name by modifying the AtomCollection based on the
    /// instructions provided by the capture name on the Node.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the capture, starting with `@`.
    /// * `node` - The node that matches the capture in the syntax tree.
    /// * `predicates` - The query predicates that modify the formatting behavior for the capture.
    ///
    /// # Errors
    ///
    /// This function returns an error if the capture name requires a predicate that is not present.
    /// It also returns an error if the capture name is not recognized by Topiary.
    // NOTE: During processing Topiary applies the function below on every match
    // of every query. As such, this function may get called multiple times
    // per node. This means that any performance loss in this function has a
    // order of magnitute higher impact than any other of such losses. We must
    // therefore ensure that the function below is as performant as we can make
    // it (within reason).
    pub fn resolve_capture(
        &mut self,
        name: &str,
        node: &Node,
        predicates: &QueryPredicates,
    ) -> FormatterResult<()> {
        log::debug!("Resolving {name}");

        let mut requires_injection_language = || {
            self.injection_language.take().ok_or_else(|| {
                FormatterError::Query(format!("@{name} requires an @injection.language tag"), None)
            })
        };

        let requires_delimiter = || {
            predicates.delimiter.as_deref().ok_or_else(|| {
                FormatterError::Query(format!("@{name} requires a #delimiter! predicate"), None)
            })
        };
        let requires_scope_id = || {
            predicates.scope_id.as_deref().ok_or_else(|| {
                FormatterError::Query(format!("@{name} requires a #scope_id! predicate"), None)
            })
        };

        // For the {prepend/append}_scope_{begin/end} captures we need this information,
        // instead of creating it for both branches, create them once here.
        let scope_information_prepend = || -> FormatterResult<ScopeInformation> {
            Ok(ScopeInformation {
                line_number: node.start_position().row(),
                scope_id: requires_scope_id()?.to_owned(),
            })
        };
        let scope_information_append = || -> FormatterResult<ScopeInformation> {
            Ok(ScopeInformation {
                line_number: node.end_position().row(),
                scope_id: requires_scope_id()?.to_owned(),
            })
        };

        let mut is_multi_line = false;
        if let Some(parent) = node.parent() {
            let parent_id = parent.id();
            if self.multi_line_nodes.contains(&parent_id) {
                is_multi_line = true;
            }
        }
        if is_multi_line && predicates.single_line_only {
            log::debug!("Skipping because context is multi-line and #single_line_only! is set");
            return Ok(());
        }
        if !is_multi_line && predicates.multi_line_only {
            log::debug!("Skipping because context is single-line and #multi_line_only! is set");
            return Ok(());
        }
        if let Some(parent_id) = self.parent_leaf_nodes.get(&node.id())
            && *parent_id != node.id()
        {
            log::debug!(
                "Skipping because the match occurred below a leaf node: {}",
                node.display_one_based()
            );
            return Ok(());
        }

        match name {
            "injection.language" => {
                let language = requires_injection_language()?;
                self.injection_content.insert((node.id(), language));
            }
            "injection.content" => {}
            "allow_blank_line_before" => {
                if self.blank_lines_before.contains(&node.id()) {
                    self.prepend(Atom::Blankline, node, predicates);
                }
            }
            "append_delimiter" => self.append(
                Atom::Literal(requires_delimiter()?.to_string()),
                node,
                predicates,
            ),
            "append_empty_softline" => {
                self.append(Atom::Softline { spaced: false }, node, predicates);
            }
            "append_hardline" => self.append(Atom::Hardline, node, predicates),
            "append_indent_start" => self.append(Atom::IndentStart, node, predicates),
            "append_indent_end" => self.append(Atom::IndentEnd, node, predicates),
            "append_input_softline" => {
                let space = if self.line_break_after.contains(&node.id()) {
                    Atom::Hardline
                } else {
                    Atom::Space
                };

                self.append(space, node, predicates);
            }
            "append_space" => self.append(Atom::Space, node, predicates),
            "append_antispace" => self.append(Atom::Antispace, node, predicates),
            "append_spaced_softline" => {
                self.append(Atom::Softline { spaced: true }, node, predicates);
            }
            "prepend_delimiter" => self.prepend(
                Atom::Literal(requires_delimiter()?.to_string()),
                node,
                predicates,
            ),
            "prepend_empty_softline" => {
                self.prepend(Atom::Softline { spaced: false }, node, predicates);
            }
            "prepend_hardline" => self.prepend(Atom::Hardline, node, predicates),
            "prepend_indent_start" => self.prepend(Atom::IndentStart, node, predicates),
            "prepend_indent_end" => self.prepend(Atom::IndentEnd, node, predicates),
            "prepend_input_softline" => {
                let space = if self.line_break_before.contains(&node.id()) {
                    Atom::Hardline
                } else {
                    Atom::Space
                };

                self.prepend(space, node, predicates);
            }
            "prepend_space" => self.prepend(Atom::Space, node, predicates),
            "prepend_antispace" => self.prepend(Atom::Antispace, node, predicates),
            "prepend_spaced_softline" => {
                self.prepend(Atom::Softline { spaced: true }, node, predicates);
            }
            // Skip over leaves
            "leaf" => {
                self.prepend(Atom::CaseBegin(Capitalisation::Pass), node, predicates);
                self.append(Atom::CaseEnd, node, predicates);
            }
            // Deletion
            "delete" => {
                self.prepend(Atom::DeleteBegin, node, predicates);
                self.append(Atom::DeleteEnd, node, predicates);
            }
            "upper_case" => {
                self.prepend(Atom::CaseBegin(Capitalisation::UpperCase), node, predicates);
                self.append(Atom::CaseEnd, node, predicates);
            }
            "lower_case" => {
                self.prepend(Atom::CaseBegin(Capitalisation::LowerCase), node, predicates);
                self.append(Atom::CaseEnd, node, predicates);
            }
            // Scope manipulation
            "prepend_begin_scope" => {
                self.prepend(
                    Atom::ScopeBegin(scope_information_prepend()?),
                    node,
                    predicates,
                );
            }
            "append_begin_scope" => {
                self.append(
                    Atom::ScopeBegin(scope_information_append()?),
                    node,
                    predicates,
                );
            }
            "prepend_end_scope" => {
                self.prepend(
                    Atom::ScopeEnd(scope_information_prepend()?),
                    node,
                    predicates,
                );
            }
            "append_end_scope" => {
                self.append(
                    Atom::ScopeEnd(scope_information_append()?),
                    node,
                    predicates,
                );
            }
            "prepend_begin_measuring_scope" => {
                self.prepend(
                    Atom::MeasuringScopeBegin(scope_information_prepend()?),
                    node,
                    predicates,
                );
            }
            "append_begin_measuring_scope" => {
                self.append(
                    Atom::MeasuringScopeBegin(scope_information_append()?),
                    node,
                    predicates,
                );
            }
            "prepend_end_measuring_scope" => {
                self.prepend(
                    Atom::MeasuringScopeEnd(scope_information_prepend()?),
                    node,
                    predicates,
                );
            }
            "append_end_measuring_scope" => {
                self.append(
                    Atom::MeasuringScopeEnd(scope_information_append()?),
                    node,
                    predicates,
                );
            }
            // Scoped softlines
            "append_empty_scoped_softline" => {
                let id = self.next_id();
                self.append(
                    Atom::ScopedSoftline {
                        id,
                        scope_id: requires_scope_id()?.to_string(),
                        spaced: false,
                    },
                    node,
                    predicates,
                );
            }
            "append_spaced_scoped_softline" => {
                let id = self.next_id();
                self.append(
                    Atom::ScopedSoftline {
                        id,
                        scope_id: requires_scope_id()?.to_string(),
                        spaced: true,
                    },
                    node,
                    predicates,
                );
            }
            "prepend_empty_scoped_softline" => {
                let id = self.next_id();
                self.prepend(
                    Atom::ScopedSoftline {
                        id,
                        scope_id: requires_scope_id()?.to_string(),
                        spaced: false,
                    },
                    node,
                    predicates,
                );
            }
            "prepend_spaced_scoped_softline" => {
                let id = self.next_id();
                self.prepend(
                    Atom::ScopedSoftline {
                        id,
                        scope_id: requires_scope_id()?.to_string(),
                        spaced: true,
                    },
                    node,
                    predicates,
                );
            }
            // Mark a leaf to be printed on an single line, with no indentation
            "single_line_no_indent" => {
                for a in &mut self.atoms {
                    if let Atom::Leaf {
                        id,
                        single_line_no_indent,
                        ..
                    } = a
                        && *id == node.id()
                    {
                        *single_line_no_indent = true;
                    }
                }

                self.append(Atom::Hardline, node, predicates);
            }
            // Mark a leaf to have all its lines be indented
            "multi_line_indent_all" => {
                for a in &mut self.atoms {
                    if let Atom::Leaf {
                        id,
                        multi_line_indent_all,
                        ..
                    } = a
                        && *id == node.id()
                    {
                        *multi_line_indent_all = true;
                    }
                }
            }
            // Mark a leaf to disable trimming
            "keep_whitespace" => {
                for a in &mut self.atoms {
                    if let Atom::Leaf {
                        id,
                        keep_whitespace,
                        ..
                    } = a
                        && *id == node.id()
                    {
                        *keep_whitespace = true;
                    }
                }
            }
            // Return a query parsing error on unknown capture names
            unknown => {
                return Err(FormatterError::Query(
                    format!("@{unknown} is not a valid capture name"),
                    None,
                ));
            }
        }

        Ok(())
    }

    /// After query processing is done, a flattened/expanded vector of atoms can be created.
    pub fn apply_prepends_and_appends(&mut self) {
        let mut expanded: Vec<Atom> = Vec::new();

        // We sort the prepends/appends so that:
        // * BeginScope(s) will always be the first element(s)
        // * MeasuringScopeBegin(s) will always come just after
        // * EndScope(s) will always be the last element(s)
        // * MeasuringScopeEnd(s) will always come just before
        // This permits proper processing of measuring scopes and scoped atoms
        // that are added at the same place as Begin/EndScopes.
        fn atom_key(atom: &Atom) -> i8 {
            match atom {
                Atom::ScopeBegin(_) => -2,
                Atom::MeasuringScopeBegin(_) => -1,
                Atom::MeasuringScopeEnd(_) => 1,
                Atom::ScopeEnd(_) => 2,
                _ => 0,
            }
        }

        for atom in &mut self.atoms {
            if let Atom::Leaf { id, .. } = atom {
                let prepends = self.prepend.entry(*id).or_default();
                prepends.sort_by_key(atom_key);
                let appends = self.append.entry(*id).or_default();
                appends.sort_by_key(atom_key);

                // Rather than cloning the atom from the old vector, we
                // simply take it. This will leave a default (empty) atom
                // in its place.
                let swapped_atom = mem::take(atom);

                if !prepends.is_empty() {
                    log::debug!("Applying prepend of {prepends:?} to {:?}.", &swapped_atom);
                }
                if !appends.is_empty() {
                    log::debug!("Applying append of {appends:?} to {:?}.", &swapped_atom);
                }

                expanded.append(prepends);
                expanded.push(swapped_atom);

                expanded.append(appends);
            } else {
                log::debug!("Not a leaf: {atom:?}");
                expanded.push(mem::take(atom));
            }
        }

        self.atoms = expanded;
    }

    /// Marks the provided node as the parent of all its child nodes by adding
    /// it to the `parent_leaf_nodes` HashMap.
    fn mark_leaf_parent(&mut self, node: &Node, parent_id: usize) {
        self.parent_leaf_nodes.insert(node.id(), parent_id);
        for child in node.children(&mut node.walk()) {
            self.mark_leaf_parent(&child, parent_id);
        }
    }

    /// This function collects the leaf nodes of a tree-sitter CST tree and stores them in the `atoms` field of the internal formatter state.
    /// It also marks the leaf parent of each node in the `leaf_parent` field of the formatter.
    /// A leaf node is either a node with no children or a node that is specified as a leaf node by the formatter.
    /// A leaf parent is the closest ancestor of a leaf node.
    ///
    /// # Arguments
    ///
    /// * `node` - The current node to process.
    /// * `source` - The full source code as a byte slice.
    /// * `parent_ids` - A vector of node ids that are the ancestors of the current node.
    /// * `level` - The depth of the current node in the CST tree.
    ///
    /// # Errors
    ///
    /// This function returns an error if it fails to convert the source code belonging to the node to UTF-8.
    fn collect_leaves_inner(
        &mut self,
        node: &Node,
        source: &[u8],
        parent_ids: &[usize],
        level: usize,
    ) -> FormatterResult<()> {
        let id = node.id();
        let parent_ids = [parent_ids, &[id]].concat();

        log::debug!(
            "CST node: {}{} - Named: {}",
            "  ".repeat(level),
            node.display_one_based(),
            node.is_named()
        );

        if node.end_byte() == node.start_byte() {
            log::debug!("Skipping zero-byte node: {}", node.display_one_based());
        } else if node.child_count() == 0
            || self.specified_leaf_nodes.contains(&node.id())
            // We treat error nodes as leaves when `tolerate_parsing_errors` is set to true.
            // This ensures Topiary does not try to further apply transformations on them.
            // If `tolerate_parsing_errors` is set to false, this part of the code is only reached if the tree contains no ERROR nodes,
            // and as such the check below would be redundant.
            || node.kind() == "ERROR"
        {
            self.atoms.push(Atom::Leaf {
                content: String::from(node.utf8_text(source)?),
                id,
                original_position: node.start_position().into(),
                single_line_no_indent: false,
                multi_line_indent_all: false,
                keep_whitespace: false,
                capitalisation: Capitalisation::Pass,
            });
            // Mark all sub-nodes as having this node as a "leaf parent"
            self.mark_leaf_parent(node, node.id());
        } else {
            for child in node.children(&mut node.walk()) {
                self.collect_leaves_inner(&child, source, &parent_ids, level + 1)?;
            }
        }

        Ok(())
    }

    /// Prepend an atom to the first leaf node in the subtree of a given node.
    ///
    /// # Arguments
    ///
    /// * `atom` - The atom to prepend.
    /// * `node` - The node to which the atom is prepended.
    /// * `predicates` - The query predicates to wrap the atom with.
    fn prepend(&mut self, atom: Atom, node: &Node, predicates: &QueryPredicates) {
        let atom = self.expand_multiline(atom, node);
        let atom = self.wrap(atom, predicates);
        // TODO: Pre-populate these
        let target_node = self.first_leaf(node);

        log::debug!(
            "Prepending {atom:?} to node {}",
            target_node.display_one_based()
        );

        self.prepend.entry(target_node.id()).or_default().push(atom);
    }

    /// Append an atom to the last leaf node in the subtree of a given node.
    ///
    /// # Arguments
    ///
    /// * `atom` - The atom to append.
    /// * `node` - The node to which the atom is appended.
    /// * `predicates` - The query predicates to wrap the atom with.
    fn append(&mut self, atom: Atom, node: &Node, predicates: &QueryPredicates) {
        let atom = self.expand_multiline(atom, node);
        let atom = self.wrap(atom, predicates);
        let target_node = self.last_leaf(node);

        log::debug!(
            "Appending {atom:?} to node {}",
            target_node.display_one_based()
        );

        self.append.entry(target_node.id()).or_default().push(atom);
    }

    /// Expands a softline atom to a hardline, space or empty atom depending on
    /// if we are in a multiline context or not.
    ///
    /// If the node's parent is labelled as a multi-line node, it expands the
    /// softline to a hardline.
    /// If the node's parent is not labelled as such, but the Atom is labelled
    /// as `spaced`, the Atom is turned into a space. The Atom is discarded if it is not
    /// labelled as `spaced`.
    /// If the node has no parent, the softline atom is discarded by returning an empty atom.
    ///
    /// The function ignores all atoms that are not softlines.
    ///
    /// # Arguments
    ///
    /// * `atom` - An atom to be expanded.
    /// * `node` - The node to which the atom applies.
    ///
    /// # Returns
    ///
    /// A new atom after expanding the softline if applicable.
    fn expand_multiline(&self, atom: Atom, node: &Node) -> Atom {
        if let Atom::Softline { spaced } = atom {
            if let Some(parent) = node.parent() {
                let parent_id = parent.id();

                if self.multi_line_nodes.contains(&parent_id) {
                    log::debug!(
                        "Expanding softline to hardline in node {} with parent {}: {}",
                        node.display_one_based(),
                        parent_id,
                        parent.display_one_based()
                    );
                    Atom::Hardline
                } else if spaced {
                    log::debug!(
                        "Expanding softline to space in node {} with parent {}: {}",
                        node.display_one_based(),
                        parent_id,
                        parent.display_one_based()
                    );
                    Atom::Space
                } else {
                    Atom::Empty
                }
            } else {
                Atom::Empty
            }
        } else {
            atom
        }
    }

    /// This function expands `ScopedSoftline` atoms depending on whether the context
    /// containing them is multiline.
    /// It does two passes over the atom collection: the first one associates each `ScopedSoftline`
    /// to its scope, and decides what to replace them with when the scope ends.
    /// The second pass applies the modifications to the atoms.
    fn post_process_scopes(&mut self) {
        type ScopeId = String;
        type LineIndex = u32;
        type ScopedNodeId = usize;
        type OpenedScopeInfo<'a> = (LineIndex, Vec<&'a Atom>, Option<bool>);
        // `opened_scopes` maintains stacks of opened scopes.
        // For each scope, we record:
        // * the line at which they started (LineIndex),
        // * the list of `ScopedSoftline` and `ScopedConditional` they contain (Vec<&Atom>),
        // * if they contain a measuring scope, whether it is multi-line (Option<bool>).
        let mut opened_scopes: HashMap<&ScopeId, Vec<OpenedScopeInfo>> = HashMap::new();
        // `opened_measuring_scopes` maintains stacks of opened measuring scopes,
        // and the line at which they started.
        let mut opened_measuring_scopes: HashMap<&ScopeId, Vec<LineIndex>> = HashMap::new();
        // We can't process `ScopedSoftline` in-place as we encounter them in the list of
        // atoms: we need to know when their encompassing scope ends to decide what to
        // replace them with. Instead of in-place modifications, we associate a replacement
        // atom to each `ScopedSoftline` atom (identified by their `id` field), then apply
        // the modifications in a second pass over the atoms.
        let mut modifications: HashMap<ScopedNodeId, Atom> = HashMap::new();
        // `force_apply_modifications` keeps track of whether something has gone wrong in the
        // post-processing (e.g. closing an unopened scope, finding a scoped atom outside
        // of its scope). If we detect any error, we don't skip the "Apply modifications" part
        // of the processing, even if the `modifications` map is empty. This is to ensure we will
        // get rid of misplaced scoped atoms.
        let mut force_apply_modifications = false;

        for atom in &self.atoms {
            if let Atom::ScopeBegin(ScopeInformation {
                line_number: line_start,
                scope_id,
            }) = atom
            {
                opened_scopes
                    .entry(scope_id)
                    .or_default()
                    .push((*line_start, Vec::new(), None));
            } else if let Atom::ScopeEnd(ScopeInformation {
                line_number: line_end,
                scope_id,
            }) = atom
            {
                if let Some((line_start, atoms, measuring_scope)) =
                    opened_scopes.get_mut(scope_id).and_then(Vec::pop)
                {
                    let multiline = if let Some(mult) = measuring_scope {
                        mult
                    } else {
                        line_start != *line_end
                    };
                    for atom in atoms {
                        if let Atom::ScopedSoftline { id, spaced, .. } = atom {
                            let new_atom = if multiline {
                                Atom::Hardline
                            } else if *spaced {
                                Atom::Space
                            } else {
                                Atom::Empty
                            };
                            modifications.insert(*id, new_atom);
                        } else if let Atom::ScopedConditional {
                            id,
                            atom,
                            condition,
                            ..
                        } = atom
                        {
                            let multiline_only = *condition == ScopeCondition::MultiLineOnly;
                            let new_atom = if multiline == multiline_only {
                                atom.deref().clone()
                            } else {
                                Atom::Empty
                            };
                            modifications.insert(*id, new_atom);
                        }
                    }
                } else {
                    log::warn!("Closing unopened scope {scope_id:?}");
                    force_apply_modifications = true;
                }
            // Open measuring scope
            } else if let Atom::MeasuringScopeBegin(ScopeInformation {
                line_number: line_start,
                scope_id,
            }) = atom
            {
                if opened_scopes.entry(scope_id).or_default().is_empty() {
                    log::warn!(
                        "Opening measuring scope with no associated regular scope {scope_id:?}"
                    );
                    force_apply_modifications = true;
                } else {
                    opened_measuring_scopes
                        .entry(scope_id)
                        .or_default()
                        .push(*line_start)
                }
            // Close measuring scope and register multi-line-ness in the appropriate regular scope
            } else if let Atom::MeasuringScopeEnd(ScopeInformation {
                line_number: line_end,
                scope_id,
            }) = atom
            {
                if let Some(line_start) =
                    opened_measuring_scopes.get_mut(scope_id).and_then(Vec::pop)
                {
                    let multi_line = line_start != *line_end;
                    if let Some((regular_line_start, vec, measuring_scope)) =
                        opened_scopes.get_mut(scope_id).and_then(Vec::pop)
                    {
                        if measuring_scope.is_none() {
                            opened_scopes.entry(scope_id).or_default().push((
                                regular_line_start,
                                vec,
                                Some(multi_line),
                            ));
                        } else {
                            log::warn!(
                                "Found several measuring scopes in a single regular scope {scope_id:?}"
                            );
                            force_apply_modifications = true;
                        }
                    } else {
                        log::warn!("Found measuring scope outside of regular scope {scope_id:?}");
                        force_apply_modifications = true;
                    }
                } else {
                    log::warn!("Closing unopened measuring scope {scope_id:?}");
                    force_apply_modifications = true;
                }
            // Register the ScopedSoftline in the correct scope
            } else if let Atom::ScopedSoftline { scope_id, .. } = atom {
                if let Some((_, vec, _)) =
                    opened_scopes.get_mut(&scope_id).and_then(|v| v.last_mut())
                {
                    vec.push(atom);
                } else {
                    log::warn!("Found scoped softline {atom:?} outside of its scope");
                    force_apply_modifications = true;
                }
            // Register the ScopedConditional in the correct scope
            } else if let Atom::ScopedConditional { scope_id, .. } = atom {
                if let Some((_, vec, _)) =
                    opened_scopes.get_mut(&scope_id).and_then(|v| v.last_mut())
                {
                    vec.push(atom);
                } else {
                    log::warn!("Found scoped conditional {atom:?} outside of its scope");
                    force_apply_modifications = true;
                }
            }
        }
        let mut still_opened: Vec<&String> = opened_scopes
            .into_iter()
            .filter_map(|(scope_id, vec)| if vec.is_empty() { None } else { Some(scope_id) })
            .collect();
        if !still_opened.is_empty() {
            log::warn!("Some scopes have been left opened: {still_opened:?}");
            force_apply_modifications = true;
        }
        still_opened = opened_measuring_scopes
            .into_iter()
            .filter_map(|(scope_id, vec)| if vec.is_empty() { None } else { Some(scope_id) })
            .collect();
        if !still_opened.is_empty() {
            log::warn!("Some measuring scopes have been left opened: {still_opened:?}");
            force_apply_modifications = true;
        }

        // Remove scopes from the atom list
        for atom in &mut self.atoms {
            match atom {
                Atom::ScopeBegin(_)
                | Atom::ScopeEnd(_)
                | Atom::MeasuringScopeBegin(_)
                | Atom::MeasuringScopeEnd(_) => *atom = Atom::Empty,
                _ => {}
            }
        }

        // Apply modifications.
        // For performance reasons, skip this step if there are no modifications to make
        if !modifications.is_empty() || force_apply_modifications {
            for atom in &mut self.atoms {
                if let Atom::ScopedSoftline { id, .. } = atom {
                    if let Some(replacement) = modifications.remove(id) {
                        *atom = replacement;
                    } else {
                        log::warn!("Found scoped softline {atom:?}, but was unable to replace it.");
                        *atom = Atom::Empty;
                    }
                } else if let Atom::ScopedConditional { id, .. } = atom {
                    if let Some(replacement) = modifications.remove(id) {
                        *atom = replacement;
                    } else {
                        log::warn!(
                            "Found scoped conditional {atom:?}, but was unable to replace it."
                        );
                        *atom = Atom::Empty;
                    }
                }
            }
        }
    }

    /// Separate post_processing of Delete sections, to avoid interference with whitespace logic
    fn post_process_deletes(&mut self) {
        let mut delete_level = 0;
        for atom in &mut self.atoms {
            match atom {
                Atom::DeleteBegin => {
                    delete_level += 1;
                    *atom = Atom::Empty;
                }
                Atom::DeleteEnd => {
                    delete_level -= 1;
                    *atom = Atom::Empty;
                }
                _ => {
                    if delete_level > 0 {
                        *atom = Atom::Empty;
                    }
                }
            }
        }
        if delete_level != 0 {
            log::warn!("The number of DeleteBegin is different from the number of DeleteEnd.");
        }
    }

    /// Separate post processing of capitalisation, to avoid confusion around whitespacing.
    fn post_process_capitalization(&mut self) {
        let mut case_context: Vec<Capitalisation> = Vec::new();
        for atom in &mut self.atoms {
            match atom {
                Atom::CaseBegin(case) => {
                    case_context.push(case.clone());
                    *atom = Atom::Empty;
                }
                Atom::CaseEnd => {
                    case_context.pop();
                    *atom = Atom::Empty;
                }
                Atom::Leaf { capitalisation, .. } => {
                    *capitalisation = case_context.last().unwrap_or(&Capitalisation::Pass).clone()
                }
                _ => {}
            }
        }
    }

    /// This function merges the spaces, new lines and blank lines.
    /// If there are several tokens of different kind one after the other,
    /// the blank line is kept over the new line which itself is kept over the space.
    /// Furthermore, this function put the indentation delimiters before any space/line atom.
    pub fn post_process(&mut self) {
        self.post_process_scopes();
        self.post_process_deletes();
        self.post_process_capitalization();
        self.post_process_inner();

        // We have taken care of spaces following an antispace. Now fix the
        // preceding spaces.
        collapse_spaces_before_antispace(&mut self.atoms);

        // We have to do one more post-processing pass, as the collapsing of
        // antispaces may have produced more empty atoms.
        self.post_process_inner();

        log::debug!("List of atoms after post-processing: {:?}", self.atoms);
    }

    /// This function post-processes the atoms in the collection.
    /// It modifies the collection in-place, removing unnecessary atoms and adjusting the position of others.
    fn post_process_inner(&mut self) {
        // A mutable reference to the atoms in the collection.
        let mut remaining = &mut self.atoms[..];

        // The previous atom in the collection. Initialized to the first atom, if it exists.
        // We a mutable reference to the previous atom so that we can modify it in-place.
        // This atoms holds the last atom encountered that is not empty.
        let mut prev: &mut Atom = if let [head, tail @ ..] = remaining {
            remaining = tail;
            head
        } else {
            return;
        };

        // Set all leading whitespace atoms to empty.
        while let Atom::Space | Atom::Antispace | Atom::Hardline | Atom::Blankline = *prev {
            *prev = Atom::Empty;
            if let [head, tail @ ..] = remaining {
                prev = head;
                remaining = tail;
            } else {
                return;
            }
        }

        // Process the remaining atoms in the collection.
        while !remaining.is_empty() {
            match (prev, remaining) {
                // If an antispace atom is followed by a space or another antispace, remove the following atom.
                (
                    moved_prev @ Atom::Antispace,
                    [head @ (Atom::Space | Atom::Antispace), tail @ ..],
                ) => {
                    *head = Atom::Empty;

                    prev = moved_prev;
                    remaining = tail;
                }
                // If two whitespace atoms follow each other, remove the non-dominant one.
                (
                    moved_prev @ (Atom::Space | Atom::Hardline | Atom::Blankline),
                    [
                        head @ (Atom::Space | Atom::Hardline | Atom::Blankline),
                        tail @ ..,
                    ],
                ) => {
                    if head.dominates(moved_prev) {
                        *moved_prev = Atom::Empty;
                    } else {
                        *head = Atom::Empty;
                    }

                    prev = moved_prev;
                    remaining = tail;
                }
                // If a whitespace or antispace atom is followed by an indent atom, swap their positions.
                (
                    moved_prev @ (Atom::Antispace | Atom::Space | Atom::Hardline | Atom::Blankline),
                    moved_remaining @ [Atom::IndentStart | Atom::IndentEnd, ..],
                ) => {
                    let old_prev = moved_prev.clone();
                    let indent = moved_remaining.first_mut().unwrap();
                    *moved_prev = indent.clone();
                    *indent = old_prev;

                    prev = moved_prev;
                    remaining = moved_remaining;
                }
                // If the current atom is not empty, update the previous atom.
                (moved_prev, [head, tail @ ..]) => {
                    prev = if matches!(head, Atom::Empty) {
                        moved_prev
                    } else {
                        head
                    };
                    remaining = tail;
                }
                // This case should never be reached because we check that `remaining` is not empty at the start of the loop.
                (_, []) => unreachable!("remaining cannot be empty"),
            }
        }
    }

    fn next_id(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }

    /// Returns the first leaf node of a given node's subtree.
    ///
    /// This function iteratively traverses the tree from the given node
    /// and returns the first leaf.
    /// A leaf node is a node that either has no children, or is specified in
    /// the `specified_leaf_nodes` HashSet.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to a node in the tree.
    ///
    /// # Returns
    ///
    /// A `Cow` enum that wraps a borrowed node.
    fn first_leaf<'tree, 'node: 'tree>(&self, node: &'node Node<'tree>) -> Cow<'node, Node<'tree>> {
        let mut node = Cow::Borrowed(node);
        while node.child_count() != 0 && !self.specified_leaf_nodes.contains(&node.id()) {
            node = Cow::Owned(node.child(0).unwrap());
        }
        node
    }

    /// Returns the last leaf node of a given node's subtree.
    ///
    /// This function iteratively traverses the tree from the given node
    /// and returns the last leaf.
    /// A leaf node is a node that either has no children, or is specified in
    /// the `specified_leaf_nodes` HashSet.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to a node in the tree.
    ///
    /// # Returns
    ///
    /// A `Cow` enum that wraps a borrowed node.
    fn last_leaf<'tree, 'node: 'tree>(&self, node: &'node Node<'tree>) -> Cow<'node, Node<'tree>> {
        let mut node = Cow::Borrowed(node);
        while node.child_count() != 0 && !self.specified_leaf_nodes.contains(&node.id()) {
            node = Cow::Owned(node.child(node.child_count() - 1).unwrap());
        }
        node
    }
}

#[derive(Clone, Debug, Default)]
/// A struct that represents a set of predicates for a query that are relevant for Topiary.
pub struct QueryPredicates {
    /// The predicate used to associate a user-defined delimiter to the
    /// `@append_delimiter` and `@prepend_delimiter` directives.
    pub delimiter: Option<String>,
    /// The predicate used to name a scope indicated by `@begin_scope` and `@end_scope`.
    pub scope_id: Option<String>,
    /// The flag that indicates whether the query only matches single-line nodes.
    pub single_line_only: bool,
    /// The flag that indicates whether the query only matches multi-line nodes.
    pub multi_line_only: bool,
    /// The flag that indicates that the query only triggers if the associated
    /// custom scope containing the matched nodes is single-line.
    pub single_line_scope_only: Option<String>,
    /// The flag that indicates that the query only triggers if the associated
    /// custom scope containing the matched nodes is multi-line.
    pub multi_line_scope_only: Option<String>,
    /// A query name, for debugging/logging purposes
    pub query_name: Option<String>,
}

/// Collapses spaces before antispace atoms in a vector of atoms.
///
/// This function modifies the given vector of atoms in place, replacing any
/// space atoms that precede an antispace atom with empty atoms.
///
/// # Arguments
///
/// * `v` - A mutable reference to a vector of atoms.
///
fn collapse_spaces_before_antispace(v: &mut [Atom]) {
    let mut antispace_mode = false;

    for a in v.iter_mut().rev() {
        if *a == Atom::Antispace {
            *a = Atom::Empty;
            antispace_mode = true;
        } else if *a == Atom::Space && antispace_mode {
            *a = Atom::Empty;
        } else if *a != Atom::Empty && *a != Atom::IndentStart && *a != Atom::IndentEnd {
            // Don't change mode when encountering Empty or Indent atoms
            antispace_mode = false;
        }
    }
}

/// Flatten the tree, depth-first, into a vector of nodes.
///
/// This function takes a reference to a node and returns a vector of references
/// to all the nodes in the tree rooted at that node, in depth-first order.
///
/// # Arguments
///
/// * `node` - A reference to a node in the tree.
///
/// # Returns
///
/// A vector of references to nodes in the provides node's subtree.
///
/// # Notes
///
/// This function uses an iterative approach instead of a recursive one for performance reasons.
/// See https://github.com/topiary/topiary/pull/417#issuecomment-1499085230 for more details.
fn dfs_flatten<'tree>(node: &Node<'tree>) -> Vec<Node<'tree>> {
    // Flatten the tree, depth-first, into a vector of nodes
    let mut walker = node.walk();
    let mut dfs_nodes = Vec::new();

    'walk: loop {
        dfs_nodes.push(walker.node());

        if !walker.goto_first_child() {
            while !walker.goto_next_sibling() {
                if !walker.goto_parent() {
                    break 'walk;
                }
            }
        }
    }

    dfs_nodes
}

/// Detects multi-line nodes in a vector of nodes and returns a set of their ids.
///
/// This function takes a slice of `Node`s that represents the nodes in a depth-first search
/// order of a syntax tree and iterates over them. For each node, it compares the start and end
/// line numbers and checks if they are different. If they are, it means the node spans multiple
/// lines and its id is added to the returned set.
///
/// # Arguments
///
/// * `dfs_nodes` - A slice of nodes in depth-first search order.
///
/// # Returns
///
/// A `HashSet` containing the ids of the multi-line nodes.
fn detect_multi_line_nodes(dfs_nodes: &[Node]) -> HashSet<usize> {
    dfs_nodes
        .iter()
        .filter_map(|node| {
            let start_line = node.start_position().row();
            let end_line = node.end_position().row();

            if end_line > start_line {
                log::debug!(
                    "Multi-line node {}: {}",
                    node.id(),
                    node.display_one_based()
                );
                return Some(node.id());
            }

            None
        })
        .collect()
}

/// Detects line breaks between nodes in a flattened vector of nodes.
///
/// This function takes a slice of nodes that have been flattened by a depth-first search (DFS)
/// and returns a struct that contains two sets of node IDs: one for the nodes that have a line break
/// before them, and one for the nodes that have a line break after them.
///
/// # Arguments
///
/// * `dfs_nodes` - A slice of nodes that have been flattened by a DFS.
/// * `minimum_line_breaks` - The minimum number of line breaks that must exist between two adjacent nodes
///   in order for them to be considered as having a line break before or after them.
///
/// # Returns
///
/// A `NodesWithLinebreaks` struct that contains two sets of node IDs: one for the nodes that have a line break
/// before them, and one for the nodes that have a line break after them.
fn detect_line_breaks(dfs_nodes: &[Node], minimum_line_breaks: u32) -> NodesWithLinebreaks {
    // Zip the flattened vector with its own tail => Iterator of pairs of adjacent nodes
    // Filter this by the threshold distance between pair components
    // Unzip into "nodes with spaces before" and "after" sets, respectively
    let (before, after) = dfs_nodes
        .iter()
        .zip(dfs_nodes[1..].iter())
        .filter_map(|(left, right)| {
            let last = left.end_position().row();
            let next = right.start_position().row();

            if next >= last + minimum_line_breaks {
                log::debug!(
                    "There are at least {} line breaks between {:?} and {:?}",
                    minimum_line_breaks,
                    left.id(),
                    right.id()
                );

                // Our node pairs look like this, hence the flip:
                //
                //          /------ before -----\
                //   <Left> <Line Breaks> <Right>
                //   \----- after ------/
                //
                // NOTE We only need the node IDs
                return Some((right.id(), left.id()));
            }

            None
        })
        .unzip();

    NodesWithLinebreaks { before, after }
}

/// So that we can easily extract the atoms using `&atom_collection[..]`
impl<Idx> std::ops::Index<Idx> for AtomCollection
where
    Idx: std::slice::SliceIndex<[Atom]>,
{
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.atoms[index]
    }
}

#[cfg(test)]
mod test {
    use crate::{Atom, atom_collection::AtomCollection};
    use test_log::test;

    #[test]
    fn post_process_indent_before_hardline() {
        let mut atom_collection = AtomCollection::new(vec![
            Atom::Literal("foo".into()),
            Atom::Hardline,
            Atom::IndentEnd,
            Atom::Literal("foo".into()),
        ]);

        atom_collection.post_process();

        assert_eq!(
            atom_collection.atoms,
            vec![
                Atom::Literal("foo".into()),
                Atom::IndentEnd,
                Atom::Hardline,
                Atom::Literal("foo".into()),
            ]
        );
    }

    #[test]
    fn post_process_hardline_before_hardline() {
        let mut atom_collection = AtomCollection::new(vec![
            Atom::Literal("foo".into()),
            Atom::Hardline,
            Atom::Hardline,
            Atom::Literal("foo".into()),
        ]);

        atom_collection.post_process();

        assert_eq!(
            atom_collection.atoms,
            vec![
                Atom::Literal("foo".into()),
                Atom::Hardline,
                Atom::Empty,
                Atom::Literal("foo".into()),
            ]
        );
    }

    #[test]
    fn post_process_empty_blank_hard() {
        let mut atom_collection = AtomCollection::new(vec![
            Atom::Empty,
            Atom::Blankline,
            Atom::Hardline,
            Atom::Literal("foo".into()),
        ]);

        atom_collection.post_process();

        assert_eq!(
            atom_collection.atoms,
            vec![
                Atom::Empty,
                Atom::Blankline,
                Atom::Empty,
                Atom::Literal("foo".into()),
            ]
        );
    }

    #[test]
    fn issue_549_post_process_indent_before_hardline_with_antispace_in_between() {
        let mut atom_collection = AtomCollection::new(vec![
            Atom::Literal("foo".into()),
            Atom::Hardline,
            Atom::Antispace,
            Atom::IndentEnd,
            Atom::Literal("foo".into()),
        ]);

        atom_collection.post_process();

        assert_eq!(
            atom_collection.atoms,
            vec![
                Atom::Literal("foo".into()),
                Atom::IndentEnd,
                Atom::Hardline,
                Atom::Empty,
                Atom::Literal("foo".into()),
            ]
        );
    }
}
