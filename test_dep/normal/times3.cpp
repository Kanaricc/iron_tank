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
    for(int i=0;i<10;i++){
        vec.push_back(i*i);
    }

    int a;cin>>a;
    cout<<vec[0]+a*3<<endl;



    return 0;
}