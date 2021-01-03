use crate::{EventStreamSubject, ReactiveEvent};
use reaper_high::{Action, Reaper};
use reaper_medium::{
    ActionValueChange, CommandId, HookPostCommand, HookPostCommand2, ReaProject, SectionContext,
    WindowContext,
};
use rxrust::prelude::*;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct ActionRx {
    action_invoked: EventStreamSubject<Rc<Action>>,
}

impl ActionRx {
    pub fn action_invoked(&self) -> impl ReactiveEvent<Rc<Action>> {
        self.action_invoked.borrow().clone()
    }
}

pub trait ActionRxProvider {
    fn action_rx() -> &'static ActionRx;
}

// Called by REAPER directly (using a delegate function)!
// Only for main section
pub struct ActionRxHookPostCommand<P: ActionRxProvider> {
    p: PhantomData<P>,
}

impl<P: ActionRxProvider> HookPostCommand for ActionRxHookPostCommand<P> {
    fn call(command_id: CommandId, _flag: i32) {
        let action = Reaper::get()
            .main_section()
            .action_by_command_id(command_id);
        let rx = P::action_rx();
        rx.action_invoked.borrow_mut().next(Rc::new(action));
    }
}

// Called by REAPER directly (using a delegate function)!
// Processes main section only.
pub struct ActionRxHookPostCommand2<P: ActionRxProvider> {
    p: PhantomData<P>,
}

impl<P: ActionRxProvider> HookPostCommand2 for ActionRxHookPostCommand2<P> {
    fn call(
        section: SectionContext,
        command_id: CommandId,
        _: ActionValueChange,
        _: WindowContext,
        _: ReaProject,
    ) {
        if section != SectionContext::MainSection {
            return;
        }
        let reaper = Reaper::get();
        let action = reaper.main_section().action_by_command_id(command_id);
        let rx = P::action_rx();
        rx.action_invoked.borrow_mut().next(Rc::new(action));
    }
}
