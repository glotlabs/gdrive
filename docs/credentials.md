# Create Google API credentials in 50 easy steps

Google has made it really easy to create api credentials for own use, just follow these few steps:

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project (or select an existing) from the meny
3. Search for `drive api` in the search bar
4. Click to enable `Google Drive API` button
5. Click on the `Credentials` menu item
6. Click on the `Configure Consent Screen` button
7. Select `External` user type (Internal is only available for workspace subscribers)
8. Click on the `Create` button
9. Fill out the fields `App name`, `User support email`, `Developer contact information` with your information.
10. Click the `Save and continue` button. If you get `An error saving your app has occurred` try changing the project name to something unique
11. Click the `Add or remove scopes` button
12. Search for `google drive api`
13. Select the scopes `.../auth/drive` and `.../auth/drive/metadata.readonly`
14. Click the `Update` button
15. Click the `Save and continue` button
16. Click the `Add users` button
17. Add the email of the user you will use with gdrive
18. Click the `Add` button until the sidebar disappears
19. Click the `Save and continue` button
20. Click on the `Credentials` menu item again
21. Click on the `Create credentials` button in the top bar and select `OAuth client ID`
22. Select application type `Desktop app` and give a name, i.e. `gdrive cli`
23. Click on the `Create` button
24. You should be presented with a Cliend Id and Client Secret


Thats it!

Gdrive will ask for your Client Id and Client Secret when using the `gdrive account add` command.
