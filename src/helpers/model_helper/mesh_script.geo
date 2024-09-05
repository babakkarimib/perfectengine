Merge "tea_pot.stp";

Mesh.CharacteristicLengthMin = 1.0;
Mesh.CharacteristicLengthMax = 1.5;

Mesh.Algorithm = 6;

Mesh.RecombineAll = 1;
Recombine Surface "*";

Mesh 2;

Save "mesh_output.msh";
