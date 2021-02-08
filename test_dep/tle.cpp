#include <iostream>
#include <vector>
#include <fstream>
using namespace std;


vector<int> vec;
int main(){
    ios::sync_with_stdio(false);
    // ofstream fout("out.txt");
    // fout<<"???"<<endl;

    vector<int> vec;
    for(int i=0;i<1e9;i++){
        vec.push_back(i*i);
    }

    return 0;
}