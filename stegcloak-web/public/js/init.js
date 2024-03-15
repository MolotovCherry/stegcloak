// a makeshift leptos on-load event
waitForElm('body div').then((elm) => {
    init();
});

// where all the init magic happens
const init = () => {
    // nothing for now
};

// simple wait for elem fn
// https://stackoverflow.com/a/61511955/9423933
function waitForElm(selector) {
    return new Promise(resolve => {
        if (document.querySelector(selector)) {
            return resolve(document.querySelector(selector));
        }

        const observer = new MutationObserver(mutations => {
            if (document.querySelector(selector)) {
                observer.disconnect();
                resolve(document.querySelector(selector));
            }
        });

        // If you get "parameter 1 is not of type 'Node'" error, see https://stackoverflow.com/a/77855838/492336
        observer.observe(document.body, {
            childList: true,
            subtree: true
        });
    });
}
