use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use {
    Backend, BoxConstraints, Drawable, Entity, EntityComponentManager, Layout, LayoutResult,
    LayoutSystem, Rect, RenderSystem, Template, Tree, Widget, World, Theme
};

pub struct EntityId(u32);

#[derive(Default)]
pub struct WidgetManager {
    world: World,
    entity_counter: u32,
    tree: Arc<RefCell<Tree>>,
}

impl WidgetManager {
    pub fn new(renderer: RefCell<Box<Backend>>, theme: Arc<Theme>) -> Self {
        let mut world = World::new();
        let tree = Arc::new(RefCell::new(Tree::default()));

        world
            .create_system(LayoutSystem { tree: tree.clone(), theme: theme.clone() })
            .with_priority(0)
            .build();

        world
            .create_system(RenderSystem { renderer })
            .with_priority(1)
            .with_sort(|comp_a, comp_b| {
                let id_a;
                let id_b;

                if let Some(id) = comp_a.downcast_ref::<EntityId>() {
                    id_a = id;
                } else {
                    return None;
                }

                if let Some(id) = comp_b.downcast_ref::<EntityId>() {
                    id_b = id;
                } else {
                    return None;
                }

                Some(id_a.0.cmp(&id_b.0))
            }).with_filter(|comp| {
                for co in comp {
                    if let Some(_) = co.downcast_ref::<Drawable>() {
                        return true;
                    }
                }
                false
            }).build();

        WidgetManager {
            world,
            entity_counter: 0,
            tree,
        }
    }

    pub fn root(&mut self, root: Arc<Widget>) {
        fn expand(
            world: &mut World,
            tree: &Arc<RefCell<Tree>>,
            widget: Arc<Widget>,
            parent: Entity,
            entity_counter: &mut u32,
        ) -> Entity {
            let entity = {
                // add bounds and default layout

                // todo: find better place for default components
                let mut entity_builder = world
                    .create_entity()
                    .with(Rect::new(10, 10, 200, 50))
                    .with(Layout::new(Box::new(
                        |_entity: Entity,
                         _ecm: &EntityComponentManager,
                         bc: &BoxConstraints,
                         children: &[Entity],
                         children_pos: &mut HashMap<Entity, (i32, i32)>,
                         size: Option<(u32, u32)>,
                         _theme: &Arc<Theme>|
                         -> LayoutResult {
                            if let Some(size) = size {
                                children_pos.insert(children[0], (0, 0));
                                LayoutResult::Size(size)
                            } else {
                                if children.len() == 0 {
                                    return LayoutResult::Size((bc.min_width, bc.min_height));
                                }
                                LayoutResult::RequestChild(children[0], *bc)
                            }
                        },
                    ))).with(EntityId(*entity_counter));

                *entity_counter += 1;

                for property in widget.properties() {
                    entity_builder = entity_builder.with_box(property);
                }

                let entity = entity_builder.build();

                tree.borrow_mut().register_node(entity);

                entity
            };

            match widget.template() {
                Template::Single(child) => {
                    let child = expand(world, tree, child, parent, entity_counter);
                    let _result = tree.borrow_mut().append_child(entity, child);
                }
                Template::Mutli(children) => {
                    for child in children {
                        let child = expand(world, tree, child, parent, entity_counter);
                        let _result = tree.borrow_mut().append_child(entity, child);
                    }
                }
                _ => {}
            }

            entity
        }

        expand(
            &mut self.world,
            &self.tree,
            root,
            0,
            &mut self.entity_counter,
        );
    }

    pub fn run(&mut self) {
        self.world.apply_filter_and_sort();
        self.world.run();
    }
}
