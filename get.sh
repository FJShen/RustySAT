#!/bin/sh

# url from which test data is acquired
SITE="https://www.cs.ubc.ca/~hoos/SATLIB/Benchmarks/SAT/RND3SAT/"
# varying number of variables and clauses
DATA=("50-218" "75-325" "100-430" "125-538" "150-645" "175-753" "200-860" "225-960" "250-1065")

mkdir -p input
cd input

for i in "${DATA[@]}"
do
    # download test data archive
    wget -P satisfiable/${i}-archive -c ${SITE}uf${i}.tar.gz
    cd satisfiable/${i}-archive
    # extract data archive
    tar -xzvf *tar.gz
    rm -f *tar.gz
    cd ..
    mkdir -p ${i}
    # flatten the directory by moving all cnf files into a folder
    find ${i}-archive -type f -exec mv -i '{}' ${i} ';'
    # remove extracted archive file
    rm -rf ${i}-archive
    cd ..

    wget -P unsatisfiable/${i}-archive -c ${SITE}uuf${i}.tar.gz
    cd unsatisfiable/${i}-archive
    tar -xzvf *tar.gz
    rm -f *tar.gz
    cd ..
    mkdir -p ${i}
    find ${i}-archive -type f -exec mv -i '{}' ${i} ';'
    rm -rf ${i}-archive
    cd ..
done

cd ..