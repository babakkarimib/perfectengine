Merge "tea_pot.stp";

Mesh.CharacteristicLengthMin = 0.9;
Mesh.CharacteristicLengthMax = 1.0;

Mesh.Algorithm = 4;

Mesh.RecombineAll = 1;
Recombine Surface "*";

Mesh 2;

Mesh.Optimize = 1;

Save "mesh_output.msh";
