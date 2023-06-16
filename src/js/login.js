const disableLoggin = true;
const wn = require('webnative');

module.exports= function onLogin(after){
    if(disableLoggin){
        let rand = (Math.ceil(Math.random()*65535)).toString(16);
        after({username:"Anon-"+rand});
    }else{
        wn.initialise({
            permissions:{
                app:{
                    name:"Yakking Yak",
                    creator:"BoisterousCoder"
                },
                // fs: {
                //     private: [
                //         wn.path.directory("Yakking Yak", "log-saves"), 
                //         wn.path.directory("Yakking Yak", "settings")
                //     ]
                // }
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
                    after(state);
                case wn.Scenario.NotAuthorised:
                    wn.redirectToLobby(state.permissions);
                    break;
            }
        });
    }
    
}