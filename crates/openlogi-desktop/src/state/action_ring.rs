//! Actions Ring settings and editor scope for the selected device.

use openlogi_core::binding::{
    ActionRingConfig, ActionRingIcon, ActionRingLayout, ActionRingSlot, RingAction,
};

use super::{AppState, DeviceRecord};

impl AppState {
    /// Actions Ring settings for the active device, including its implicit
    /// default layout when nothing has been persisted yet.
    #[must_use]
    pub fn current_action_ring(&self) -> ActionRingConfig {
        self.current_record()
            .and_then(DeviceRecord::persistent_config_key)
            .map(|key| self.config.action_ring(key))
            .unwrap_or_default()
    }

    /// Complete layout shown by the Actions Ring editor. An application with
    /// no saved profile inherits the default layout until its first edit.
    #[must_use]
    pub fn current_action_ring_layout(&self) -> ActionRingLayout {
        let ring = self.current_action_ring();
        ring.effective_layout(self.editing_app())
    }

    /// Replace or clear one slot in the open Actions Ring layout.
    pub fn commit_action_ring_slot(&mut self, slot: ActionRingSlot, action: Option<RingAction>) {
        self.edit_action_ring_layout("Actions Ring slot", |layout| {
            layout.set_action(slot, action);
        });
    }

    /// Set or restore the action-derived icon for one slot in the open layout.
    pub fn commit_action_ring_icon(&mut self, slot: ActionRingSlot, icon: Option<ActionRingIcon>) {
        self.edit_action_ring_layout("Actions Ring icon", |layout| {
            layout.set_icon(slot, icon);
        });
    }

    fn edit_action_ring_layout(&mut self, what: &str, edit: impl FnOnce(&mut ActionRingLayout)) {
        let Some(key) = self
            .current_record()
            .and_then(DeviceRecord::persistent_config_key)
            .map(str::to_string)
        else {
            return;
        };
        let app = self.editing_app().map(str::to_string);
        self.config.edit(|config| {
            let ring = &mut config.devices.entry(key).or_default().action_ring;
            let layout = match app {
                Some(app) => {
                    let inherited = ring.default.clone();
                    ring.per_app.entry(app).or_insert(inherited)
                }
                None => &mut ring.default,
            };
            edit(layout);
        });
        self.persist_and_reload(what);
    }

    /// Enable or disable the active device's Actions Ring.
    pub fn commit_action_ring_enabled(&mut self, enabled: bool) {
        let Some(key) = self
            .current_record()
            .and_then(DeviceRecord::persistent_config_key)
            .map(str::to_string)
        else {
            return;
        };
        self.config
            .edit(|config| config.set_action_ring_enabled(&key, enabled));
        self.persist_and_reload("Actions Ring enabled state");
    }

    /// Enable or disable hover and activation haptics for the active ring.
    pub fn commit_action_ring_haptics(&mut self, enabled: bool) {
        let Some(key) = self
            .current_record()
            .and_then(DeviceRecord::persistent_config_key)
            .map(str::to_string)
        else {
            return;
        };
        self.config
            .edit(|config| config.set_action_ring_haptics(&key, enabled));
        self.persist_and_reload("Actions Ring haptics");
    }
}
