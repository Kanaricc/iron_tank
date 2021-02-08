#include <iostream>
#include <fstream>
using namespace std;

int main(){

    ofstream fout("debug.out");

    bool ok=true;
    for(int i=0;i<10;i++){
        cout<<i<<endl;
        int x;cin>>x;
        fout<<x<<endl;
        if(x!=(1<<i))ok=false;
    }

    if(ok){
        cerr<<"same"<<endl;
    }else{
        cerr<<"different"<<endl;
    }

    return 0;

}