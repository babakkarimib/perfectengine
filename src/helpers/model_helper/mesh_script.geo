Merge "tea_pot.stp";

Mesh.CharacteristicLengthMin = 0.5;
Mesh.CharacteristicLengthMax = 0.8;

Mesh.Algorithm = 6;

Mesh.RecombineAll = 1;
Recombine Surface "*";

Mesh 2;

Mesh.Optimize = 1;

Save "mesh_output.msh";
