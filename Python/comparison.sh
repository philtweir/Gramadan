#!/bin/sh

# 1. Start by getting a copy of BuNaMo (INMD)
(cd .. && git clone https://github.com/michmech/BuNaMo data) || (cd ../data && git pull)
rm -f output/data
ln -s ../../data output/data
rm -f ../Tester/data
ln -s ../data ../Tester/data

# 2. Execute the C# testing program

echo "Running C#"
cd ../Tester
# xbuild Tester.csproj
rm -f OUTPUT.txt
echo "" | ./bin/Debug/Tester.exe > OUTPUT.txt
# Ignore any errors about .NET versions. We could redirect to stderr,
# but that creates an unnecessary output-testing loophole.
sed -i "s/WARNING: The runtime version supported by this application is unavailable\.//g" OUTPUT.txt
sed -i "/Using default runtime: v[0-9.]*/d" OUTPUT.txt
cd ../Python
echo "Completed C#"

# 3. Then execute the Python tester from the root directory

echo "Running Python"
mkdir -p output && cd output
rm -f OUTPUT.txt
echo "" | (PYTHONPATH=$PYTHONPATH:.. python3 -m gramadan.tester.program) > OUTPUT.txt
cd ..
echo "Completed Python"

# 4. Finally execute the comparison tool

echo "Comparing"
pytest tests
