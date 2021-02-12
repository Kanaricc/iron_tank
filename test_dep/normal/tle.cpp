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
        vec.push_back(i);
        if(vec.size()>10)vec.pop_back();
    }

    int a;cin>>a;
    cout<<vec[0]+a*2<<endl;

    return 0;
}