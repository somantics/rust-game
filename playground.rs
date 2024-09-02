use std::any::Any;

use crate::game::components::spells::Spell;

#[derive(Debug, Clone)]
struct NewHealth {}

impl NewComp for NewHealth {}

trait NewComp {}

fn get_comp_clone<T: NewComp + Clone + 'static>(comps: &'static [&'static dyn NewComp]) -> Option<T> {
    comps
        .into_iter()
        .find_map(|comp| {
            let any = &comp as &dyn Any;
            any.downcast_ref::<T>().cloned()
        })
}

fn get_comp_copy<T: NewComp + Copy + 'static>(comps:  &'static [&'static dyn NewComp]) -> Option<T> {
    comps
        .into_iter()
        .find_map(|comp| {
            let any = &comp as &dyn Any;
            any.downcast_ref::<T>().copied()
        })
}

struct TestCompDiff {
    bool: Option<bool>,
    vec: Option<Vec<usize>>,
    spell: Option<Spell>,
}

impl Diff for TestCompDiff {
    type Item = TestComp;

}

struct TestComp {
    bool: bool,
    vec: Vec<usize>,
    spell: Spell,
}  

impl NewDiffable for TestComp {
    fn apply_diff<T: Any + Diff<Item = Self>> (&mut self, other: &T) {
        let other_any = other as &dyn Any;
        match other_any.downcast_ref::<TestCompDiff>() {
            Some(diff) => {
                if let Some(value) = diff.bool {
                    self.bool = value;
                };

                if let Some(value) = &diff.vec {
                    self.vec = value.clone();
                };
                
                if let Some(value) = &diff.spell {
                    self.spell = value.clone();
                };
            },
            None => {}
        };
    }
}

trait Diff {
    type Item;

}

trait NewDiffable {
    fn apply_diff<T: Any + Diff<Item = Self>> (&mut self, other: &T);
}