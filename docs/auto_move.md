# Auto move

## Auto crate push

<p align="center"><img src="assets/auto_crate_push.gif" width=70%></p>

This feature can significantly assist users when playing larger levels.  
At the same time, it also maintains the traditional control method.

Clicking on a crate will display all the points that the crate can reach (without moving other crates).

Take Microban #155 as an example:

<p align="center"><img src="assets/auto_crate_push_1.png" width=70%></p>

Clicking on one of the points will automatically push the selected crate to that position.

In this case, the user can click on the target, and the character will automatically push the selected crate to the target to complete the level.

Some areas where the crates are reachable do not display points. This is because pushing the crate to those positions will lead to a deadlock and the player will be unable to continue completing the level.

<p align="center"><img src="assets/auto_crate_push_2.png" width=70%></p>

## Auto player move

Click to select the player and display the player's reachable area. Click on a position in the area and the player will automatically move to that position.

<p align="center"><img src="assets/auto_player_move.png" width=70%></p>

User can also directly click on the player's reachable area without selecting the player, and the player will automatically move to that position.

## Controversial

This feature is a bit controversial, with some users saying it's akin to cheating.

For simple levels, this does significantly reduce the level difficulty. An extreme example is a level with only a single crate and target, which means the player can complete it without having to do any reasoning.

But for challenging levels, the difficulty mainly lies in the intricate pushing relationship between multiple crates, rather than the pushing of a single crate. This feature allows players to focus on more complex reasoning instead of repeating the simple work of pushing a single crate.

In addition, this feature is **optional**. Users can still use the traditional control methods.
