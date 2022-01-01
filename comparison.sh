#!/bin/sh

# 1. Start by executing the C# testing program

echo "Running C#"
cd Tester
# xbuild Tester.csproj
rm -f OUTPUT.txt
echo "" | ./bin/Debug/Tester.exe > OUTPUT.txt
# Ignore any errors about .NET versions. We could redirect to stderr,
# but that creates an unnecessary output-testing loophole.
sed -i "s/WARNING: The runtime version supported by this application is unavailable\.//g" OUTPUT.txt
sed -i "/Using default runtime: v[0-9.]*/d" OUTPUT.txt
cd ..
echo "Completed C#"

# 2. Then execute the Python tester from the root directory

echo "Running Python"
rm -f OUTPUT.txt
echo "" | python3 -m Gramadan.tester.program > OUTPUT.txt
echo "Completed Python"

# 3. Finally execute the comparison tool

echo "Comparing"
pytest test_compare.py
