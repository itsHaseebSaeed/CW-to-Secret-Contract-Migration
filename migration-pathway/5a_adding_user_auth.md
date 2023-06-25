# Adding User Authentication

Secret contracts provide computational privacy. Permissioned viewing is entirely unique to Secret Network with its encrypted state. It gives you ability to have precise control over who has access to what data. In order to determine when data is allowed to be released however, you need to have a way to prove who is trying to access it. Our two ways of doing this are through Viewing Keys and Permits/Certs.

If you want to learn more about viewing keys and permits. And how they are implemented please check our pathway '[Implementing Viewing key and Permits](https://scrt.university/pathways/33/implementing-viewing-keys-and-permits)'.

For this contract we'll restrict access to `QueryMsg::GetUserCount` and leaving global `QueryMsg::GetCount {}` open to the public.

There are two steps to adding authenticatio. First we'll add viewing keys and then Query Permits.
