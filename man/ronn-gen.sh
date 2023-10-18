# Run this scripts from base path!
PATH=$(pwd)/ronn/bin:$PATH

ronn ./man/xornet-reporter.1.ronn
rm ./man/xornet-reporter.1.html
echo "ronn man page generation done."
