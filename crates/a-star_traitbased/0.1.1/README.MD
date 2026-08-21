# A* for stuctures

This is a barebone implementation of A* that uses a structure as a base. It can be vary useful if you have already a structure that includes movement, being a game, animation, or some form of structe that is alredy able to determine possilbe path.
## Implementation:
* implement the PathGenerator train on the structure;
* call AStar::run(from_struct: Box<&T: PathGenerator>, start: (usize, usize), target: (Option<usize>, Option<usize>)) -> Option<Vec<(usize, usize)>> 
## Notes:
Expected struct should work with 2D positional arguments (x, y), that are numeric.
PathGenerator will requite the implementation of:
* generate_paths -> logic used to generate possible path from positions (here is the place to inclde road blocks and additional logic);
* calculate_cost -> logic used to derive cost of transfer from position to next position;
* calculate_heuristic_cost -> logic used to derive relative cost to the target;

AStar::run takes a target that can have either both x and y (or exact point of arival) or only one (x or y), reaching a side of the map.

AStar::run returns Option for a Vector of position leading from the target back to the start or None if there is no path available.

* This is an implemetaion that I use in a project but could be useful to other, so I decided to share it.