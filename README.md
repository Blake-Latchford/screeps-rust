#TODO
1. Make creep sub-class composition rather than inheritance.
    1. Split creeps::Creep into actor and state flow.
    1. Rename creeps module to creep.
    1. Make Creep::creep private.
1. Move spawn target prioritiziation into allocator.
    1. Make allocator derriviatves private to allocator.
    1. Move NAME_PREFIX from derivative allocators into main allocator.
        1. Remove NAME_PREFIX from creep sub classes.
1. Handle creep colision deadlock.
    1. Rewrite Creep::move_to_target to reduce nesting.
    1. Don't haul energy to spawn if spawn is full
    1. Handle move failures.
1. Relate maximum number of harvesters to adjacent open spaces.
1. Remove SpawnManager.
1. Make spawn understand extensions.
    1. Make allocators understand extesions.