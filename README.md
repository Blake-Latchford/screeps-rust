#TODO
1. Make creep sub-class composition rather than inheritance.
    1. Rename creeps module to creep.
    1. Convert Mode and Role enum string resolution to use a hash.
1. Handle creep colision deadlock.
    1. Rewrite Creep::move_to_target to reduce nesting.
    1. Don't haul energy to spawn if spawn is full
    1. Handle move failures.
1. Relate maximum number of harvesters to adjacent open spaces.
1. Remove SpawnManager.
1. Make spawn understand extensions.
1. Set worker mode based on target.
1. Stop high frequency switching of worker target.