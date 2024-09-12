use std::any::Any;

use crate::game::components::spells::Spell;

pub trait NewComp {}

pub trait Diff {
    type BaseType;
}

pub trait NewDiffable {
    fn apply_diff<T: Any + Diff<BaseType = Self>> (&mut self, other: &T);
}

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
    vec: Option<Vec<usize>>, //vecs need to be options, because there's a difference between diff with empty vec and no diff at all
    spell: Option<Spell>,
}

impl Diff for TestCompDiff {
    type BaseType = TestComp;

}

struct TestComp {
    bool: bool,
    vec: Vec<usize>,
    spell: Spell,
}  

impl NewDiffable for TestComp {
    fn apply_diff<T: Any + Diff<BaseType = Self>> (&mut self, other: &T) {
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
