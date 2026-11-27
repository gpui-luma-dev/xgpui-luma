//! Which item of a collection the pointer is over and which it holds down (#433): one tracker for control
//! group, accordion, table and tree view, instead of hover and press bookkeeping written per control.
//!
//! Every change comes back in order, an item left before the next is entered and released before the next is
//! pressed, so a control's events always say "left" before "entered".

/// One change to an item's hover or press, keyed as the control keys its items (an index or an id).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ItemChange<K> {
    Hovered { key: K, hovered: bool },
    Pressed { key: K, pressed: bool },
}

/// The hovered and pressed item of a collection.
#[derive(Clone, Debug)]
pub(crate) struct ItemTracker<K> {
    hovered: Option<K>,
    pressed: Option<K>,
}

impl<K> Default for ItemTracker<K> {
    fn default() -> Self {
        Self { hovered: None, pressed: None }
    }
}

impl<K: Clone + PartialEq> ItemTracker<K> {
    #[cfg(test)]
    pub fn hovered(&self) -> Option<&K> {
        self.hovered.as_ref()
    }

    #[cfg(test)]
    pub fn pressed(&self) -> Option<&K> {
        self.pressed.as_ref()
    }

    pub fn is_hovered(&self, key: &K) -> bool {
        self.hovered.as_ref() == Some(key)
    }

    pub fn is_pressed(&self, key: &K) -> bool {
        self.pressed.as_ref() == Some(key)
    }

    /// The pointer entered (`hovered`) or left `key`. Entering leaves the item hovered before it; leaving an
    /// item also releases a press on it. Leaving an item that is not hovered changes nothing.
    pub fn hover(&mut self, key: K, hovered: bool) -> Vec<ItemChange<K>> {
        let mut changes = Vec::new();
        if hovered {
            if self.is_hovered(&key) {
                return changes;
            }
            self.leave(&mut changes);
            self.hovered = Some(key.clone());
            changes.push(ItemChange::Hovered { key, hovered: true });
        } else if self.is_hovered(&key) {
            self.leave(&mut changes);
        }
        changes
    }

    /// Press `key`, releasing any other item first.
    pub fn press(&mut self, key: K) -> Vec<ItemChange<K>> {
        let mut changes = Vec::new();
        if self.is_pressed(&key) {
            return changes;
        }
        self.release_into(&mut changes);
        self.pressed = Some(key.clone());
        changes.push(ItemChange::Pressed { key, pressed: true });
        changes
    }

    /// Release the pressed item, if any.
    pub fn release(&mut self) -> Vec<ItemChange<K>> {
        let mut changes = Vec::new();
        self.release_into(&mut changes);
        changes
    }

    /// Leave the hovered item and release the pressed one: the control was disabled, its items replaced, or
    /// the pointer gone.
    pub fn clear(&mut self) -> Vec<ItemChange<K>> {
        let mut changes = Vec::new();
        self.leave(&mut changes);
        self.release_into(&mut changes);
        changes
    }

    /// Leave and release items `keep` no longer accepts, such as rows scrolled or collapsed away.
    pub fn retain(&mut self, keep: impl Fn(&K) -> bool) -> Vec<ItemChange<K>> {
        let mut changes = Vec::new();
        if self.hovered.as_ref().is_some_and(|key| !keep(key)) {
            self.leave(&mut changes);
        }
        if self.pressed.as_ref().is_some_and(|key| !keep(key)) {
            self.release_into(&mut changes);
        }
        changes
    }

    /// Leave the hovered item, releasing a press on it.
    fn leave(&mut self, changes: &mut Vec<ItemChange<K>>) {
        let Some(key) = self.hovered.take() else {
            return;
        };
        if self.is_pressed(&key) {
            self.release_into(changes);
        }
        changes.push(ItemChange::Hovered { key, hovered: false });
    }

    fn release_into(&mut self, changes: &mut Vec<ItemChange<K>>) {
        if let Some(key) = self.pressed.take() {
            changes.push(ItemChange::Pressed { key, pressed: false });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ItemChange::{Hovered, Pressed};
    use super::*;

    #[test]
    fn moving_between_items_leaves_before_entering() {
        let mut items = ItemTracker::default();
        assert_eq!(items.hover(1, true), vec![Hovered { key: 1, hovered: true }]);
        assert_eq!(items.hover(1, true), vec![], "entering again changes nothing");
        assert_eq!(items.hover(2, true), vec![Hovered { key: 1, hovered: false }, Hovered { key: 2, hovered: true }]);
        assert_eq!(items.hover(1, false), vec![], "a late leave of an item no longer hovered changes nothing");
        assert_eq!(items.hover(2, false), vec![Hovered { key: 2, hovered: false }]);
        assert_eq!(items.hovered(), None);
    }

    #[test]
    fn leaving_an_item_releases_its_press() {
        let mut items = ItemTracker::default();
        items.hover("a", true);
        assert_eq!(items.press("a"), vec![Pressed { key: "a", pressed: true }]);
        assert_eq!(
            items.hover("b", true),
            vec![
                Pressed { key: "a", pressed: false },
                Hovered { key: "a", hovered: false },
                Hovered { key: "b", hovered: true },
            ]
        );
        assert_eq!(items.pressed(), None);
    }

    #[test]
    fn pressing_another_item_releases_the_first() {
        let mut items = ItemTracker::default();
        items.press(1);
        assert_eq!(items.press(2), vec![Pressed { key: 1, pressed: false }, Pressed { key: 2, pressed: true }]);
        assert_eq!(items.release(), vec![Pressed { key: 2, pressed: false }]);
        assert_eq!(items.release(), vec![]);
    }

    #[test]
    fn clear_and_retain_drop_hover_then_press() {
        let mut items = ItemTracker::default();
        items.hover(1, true);
        items.press(2);
        assert_eq!(items.clear(), vec![Hovered { key: 1, hovered: false }, Pressed { key: 2, pressed: false }]);

        items.hover(1, true);
        items.press(2);
        assert_eq!(items.retain(|key| *key == 1), vec![Pressed { key: 2, pressed: false }]);
        assert!(items.is_hovered(&1));
    }
}
