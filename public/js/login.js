// import * as wn from '../libs/webnative.js'
const wn = self.webnative

export function onLogin(before, after){
    wn.initialise({
        permissions:{
            app:{
                name:"Rusty Chat",
                creator:"BoisterousCoder"
            },
            fs: {
                private: [
                    //wn.path.directory("RustyChat", "log-saves"), 
                    //wn.path.directory("RustyChat", "settings")
                ]
            }
        }
    }).catch(err => {
        console.error(err);
        switch(err){
            case wn.InitialisationError.InsecureContext:
                alert("This app was unable to setup a secure connection to the server.");
                break;
            case wn.InitialisationError.UnsupportedBrowser:
                alert("Your browser is not supported by this app.");
                break;
        }
    }).then(state=>{
        switch (state.scenario) {
            case wn.Scenario.AuthCancelled:
                alert("Authorization was cancelled. This app will not continue to load");
                break;
            case wn.Scenario.AuthSucceeded:
            case wn.Scenario.Continuation:
                before.then(() => after(state));
            case wn.Scenario.NotAuthorised:
                wn.redirectToLobby(state.permissions);
                break;
        }
    });
}