#include <iostream>
#include <fstream>
using namespace std;

int main(int argc,char* argv[]){
    ifstream input(argv[1]);
    ifstream output(argv[2]);

    string s1, s2;
    input >> s1;
    output >> s2;

    if(s1 == s2) {
        cout << "same" << endl << "" << endl;
    } else {
        cout << "different" << endl << "" << endl;
    }

    return 0;
}