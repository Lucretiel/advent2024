//! A library for doing dynamic programming in a non-recursive way

use std::{
    collections::{BTreeMap, HashMap},
    convert::Infallible,
    error::Error,
    fmt::{self, Debug, Display, Formatter},
    hash::{BuildHasher, Hash},
    marker::PhantomData,
};

pub trait SubtaskStore<K, V> {
    /// Add a new subtask solution to the store. Return the old solution, if
    /// present.
    fn add(&mut self, goal: K, solution: V) -> Option<V>;

    /// Fetch a known solution for a subtask, if possible
    fn get(&self, goal: &K) -> Option<&V>;

    /// Check if a subtask has a known solution
    fn contains(&self, goal: &K) -> bool;
}

impl<K, V, T> SubtaskStore<K, V> for &mut T
where
    T: SubtaskStore<K, V>,
{
    fn add(&mut self, goal: K, solution: V) -> Option<V> {
        T::add(*self, goal, solution)
    }

    fn get(&self, goal: &K) -> Option<&V> {
        T::get(*self, goal)
    }

    fn contains(&self, goal: &K) -> bool {
        T::contains(*self, goal)
    }
}

impl<K, V, S> SubtaskStore<K, V> for HashMap<K, V, S>
where
    K: Eq + Hash,
    S: Default + BuildHasher,
{
    fn add(&mut self, goal: K, solution: V) -> Option<V> {
        self.insert(goal, solution)
    }

    fn get(&self, goal: &K) -> Option<&V> {
        self.get(goal)
    }

    fn contains(&self, goal: &K) -> bool {
        self.contains_key(goal)
    }
}

impl<K: Ord, V> SubtaskStore<K, V> for BTreeMap<K, V> {
    fn add(&mut self, goal: K, solution: V) -> Option<V> {
        self.insert(goal, solution)
    }

    fn get(&self, goal: &K) -> Option<&V> {
        self.get(goal)
    }

    fn contains(&self, goal: &K) -> bool {
        self.contains_key(goal)
    }
}

#[derive(Debug)]
pub struct Dependency<'a, K> {
    key: K,
    lifetime: PhantomData<&'a K>,
}

#[derive(Debug)]
pub enum TaskInterrupt<'a, K, E> {
    Dependency(Dependency<'a, K>),
    Error(E),
    Tail(K),
}

impl<'a, K, E> From<Dependency<'a, K>> for TaskInterrupt<'a, K, E> {
    fn from(dep: Dependency<'a, K>) -> Self {
        TaskInterrupt::Dependency(dep)
    }
}

pub trait Subtask<Goal, Solution> {
    fn precheck(&self, goals: impl IntoIterator<Item = Goal>) -> Result<(), Dependency<'_, Goal>>;
    fn solve(&self, goal: Goal) -> Result<&Solution, Dependency<'_, Goal>>;
}

pub trait Task<Goal, Solution, Error> {
    type State;

    fn solve<'sub>(
        &self,
        goal: &Goal,
        subtasker: &'sub impl Subtask<Goal, Solution>,
        state: &mut Option<Self::State>,
    ) -> Result<Solution, TaskInterrupt<'sub, Goal, Error>>;
}

pub trait StatelessTask<Goal, Solution, Error> {
    fn solve<'sub>(
        &self,
        goal: &Goal,
        subtasker: &'sub impl Subtask<Goal, Solution>,
    ) -> Result<Solution, TaskInterrupt<'sub, Goal, Error>>;
}

impl<Goal, Solution, Error, T> Task<Goal, Solution, Error> for T
where
    T: StatelessTask<Goal, Solution, Error>,
{
    type State = Infallible;

    #[inline(always)]
    fn solve<'sub>(
        &self,
        goal: &Goal,
        subtasker: &'sub impl Subtask<Goal, Solution>,
        _state: &mut Option<Infallible>,
    ) -> Result<Solution, TaskInterrupt<'sub, Goal, Error>> {
        StatelessTask::solve(self, goal, subtasker)
    }
}

#[derive(Debug)]
pub enum DynamicError<K, E> {
    /// The solver found a circular dependency while solving
    CircularDependency(K),

    /// The solver itself returned an error
    Error(E),
}

impl<K: Debug, E> Display for DynamicError<K, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            DynamicError::CircularDependency(ref dep) => {
                write!(f, "goal {:?} has a circular dependency on itself", dep)
            }
            DynamicError::Error(..) => write!(f, "solver encountered an error"),
        }
    }
}

impl<K: Debug, E: Error + 'static> Error for DynamicError<K, E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            DynamicError::CircularDependency(..) => None,
            DynamicError::Error(ref err) => Some(err),
        }
    }
}

#[derive(Debug, Default)]
struct Subtasker<S> {
    store: S,
}

impl<K, V, S> Subtask<K, V> for Subtasker<S>
where
    S: SubtaskStore<K, V>,
{
    fn precheck(&self, goals: impl IntoIterator<Item = K>) -> Result<(), Dependency<K>> {
        goals
            .into_iter()
            .try_for_each(|goal| match self.store.contains(&goal) {
                true => Ok(()),
                false => Err(Dependency {
                    key: goal,
                    lifetime: PhantomData,
                }),
            })
    }

    fn solve(&self, goal: K) -> Result<&V, Dependency<K>> {
        self.store.get(&goal).ok_or(Dependency {
            key: goal,
            lifetime: PhantomData,
        })
    }
}

// TODO: add a mechanism to request a set of dependencies as a block

/// Solve a dynamic algorithm.
///
/// This will run task.solve(&goal, subtasker). The task can request subgoal
/// solutions by calling `subtasker.solve(subgoal)?`; this will halt the
/// function and call task.solve(&subgoal, subtasker). In this way, execute
/// performs a depth-first traversal of the problem space. Solutions to subtasks
/// are stored in the store and are provided by the subtasker to the caller
/// when available; this ensures that each subtask is solved at most once.
///
/// Note that every time a subtask is requested but not available, the ? will
/// return a dependency request from the solver. This means the solver will be
/// restarted from scratch once for each dependency it requests, until the
/// store can fulfill them all. To prevent wasting work finding a partial
/// solution, you can call `subtasker.precheck(iter)?` at the beginning of
/// your Task::solve implementation with an iterator over all the subgoal
/// dependencies you're expecting
pub fn execute<Goal, Solution, Error>(
    goal: Goal,
    task: &impl Task<Goal, Solution, Error>,
    store: impl SubtaskStore<Goal, Solution>,
) -> Result<Solution, DynamicError<Goal, Error>>
where
    Goal: PartialEq,
{
    let mut subtasker = Subtasker { store };

    // TODO: use an ordered hash map for faster circular checks
    let mut dependency_stack = vec![];
    let mut current_goal = goal;
    let mut current_state = None;

    loop {
        // NOTE: We could check if the current_goal is already in the store,
        // but it should be impossible for that to be the case at this point,
        // since the only way to add things to the store is with a Dependency,
        // and the only way to get a Dependency is if the store reports that
        // it *doesn't* already contain that solution.
        //
        // This means that the only time this could happen is if the store
        // contains the solution for the *original* goal, which we assume
        // doesn't happen.

        match task.solve(&current_goal, &subtasker, &mut current_state) {
            Ok(solution) => match dependency_stack.pop() {
                None => break Ok(solution),
                Some((dependent_goal, state)) => {
                    subtasker.store.add(current_goal, solution);
                    current_goal = dependent_goal;
                    current_state = state;
                }
            },
            Err(TaskInterrupt::Error(err)) => break Err(DynamicError::Error(err)),
            Err(TaskInterrupt::Dependency(Dependency { key: subgoal, .. })) => {
                dependency_stack.push((current_goal, current_state));
                match dependency_stack.iter().any(|(goal, ..)| *goal == subgoal) {
                    true => break Err(DynamicError::CircularDependency(subgoal)),
                    false => {
                        current_goal = subgoal;
                        current_state = Default::default();
                    }
                }
            }
            Err(TaskInterrupt::Tail(tail_goal)) => {
                match dependency_stack.iter().any(|(goal, ..)| *goal == tail_goal) {
                    true => break Err(DynamicError::CircularDependency(tail_goal)),
                    false => {
                        current_goal = tail_goal;
                        current_state = Default::default();
                    }
                }
            }
        }
    }
}
