wget -P input/satisfiable -c https://www.cs.ubc.ca/~hoos/SATLIB/Benchmarks/SAT/RND3SAT/uf50-218.tar.gz
cd input/satisfiable
tar -xzvf *tar.gz
rm *tar.gz
cd ../..

wget -P input/unsatisfiable -c https://www.cs.ubc.ca/~hoos/SATLIB/Benchmarks/SAT/RND3SAT/uuf50-218.tar.gz
cd input/unsatisfiable
tar -xzvf *tar.gz
rm *tar.gz
mv UUF50.218.1000/* .
rmdir UUF50.218.1000
cd ../..
