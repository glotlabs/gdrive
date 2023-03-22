# Create Google API credentials in 50 easy steps

Google has made it really easy to create api credentials for own use, just follow these few steps:

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project (or select an existing) from the menu [[screenshot]](https://user-images.githubusercontent.com/720405/210136984-7ed0eb00-f940-47c2-a1b7-95147e0f6ed8.png)
3. Search for `drive api` in the search bar and select `Google drive api` under the marketplace section [[screenshot]](https://user-images.githubusercontent.com/720405/210137041-57633760-eb57-4c87-bacf-a9850c363a63.png)
4. Click to enable `Google Drive API` button [[screenshot]](https://user-images.githubusercontent.com/720405/210137243-3f7c1ea6-519b-4c50-afea-577e19fe543d.png)
5. Click on the `Credentials` menu item
6. Click on the `Configure Consent Screen` button [[screenshot]](https://user-images.githubusercontent.com/720405/210137298-9c9eb3d1-9420-4bdb-bd98-4e6e778c8ed5.png)
7. Select `External` user type (Internal is only available for workspace subscribers) [[screenshot]](https://user-images.githubusercontent.com/720405/210137317-de4b8bea-235d-498d-b78d-b0c37dd96717.png)
8. Click on the `Create` button
9. Fill out the fields `App name`, `User support email`, `Developer contact information` with your information; you will need to put the Project ID into the app name (keep the other fields empty) [[screenshot]](https://user-images.githubusercontent.com/720405/210137365-09aa2294-8984-45ef-9a29-7f485cfbe7ac.png)
10. Click the `Save and continue` button. If you get `An error saving your app has occurred` try changing the project name to something unique
11. Click the `Add or remove scopes` button
12. Search for `google drive api`
13. Select the scopes `.../auth/drive` and `.../auth/drive.metadata.readonly` [[screenshot]](https://user-images.githubusercontent.com/720405/210137392-f851aa1e-ea59-4c19-885e-d246992c4dd7.png)
14. Click the `Update` button
15. Click the `Save and continue` button [[screenshot]](https://user-images.githubusercontent.com/720405/210137425-44cab632-c885-495d-bb10-3b6e842ed79a.png)
16. Click the `Add users` button
17. Add the email of the user you will use with gdrive [[screenshot]](https://user-images.githubusercontent.com/720405/210137458-ec6a6fb3-ea0c-47e8-a8ec-fe230841ba3b.png)
18. Click the `Add` button until the sidebar disappears
19. Click the `Save and continue` button [[screenshot]](https://user-images.githubusercontent.com/720405/210137468-9c1fc03e-cb18-4798-a17c-1a6c912f07a8.png)
20. Click on the `Credentials` menu item again
21. Click on the `Create credentials` button in the top bar and select `OAuth client ID` [[screenshot]](https://user-images.githubusercontent.com/720405/210137498-dc9102c4-2720-466d-809a-4d8947dbb0a0.png)
22. Select application type `Desktop app` and give a name [[screenshot]](https://user-images.githubusercontent.com/720405/210137673-d3a387ab-3bbe-4af3-81c8-7c744aed8bd5.png)
23. Click on the `Create` button
24. You should be presented with a Cliend Id and Client Secret [[screenshot]](https://user-images.githubusercontent.com/720405/210137709-587edb53-4703-4ad3-8941-6130f47d0547.png)
25. Click on `OAuth consent screen`
26. Click on `Publish app` (to prevent the token from expiring after 7 days) [[screenshot]](https://user-images.githubusercontent.com/720405/216276113-18356d78-c81c-42c1-be2b-49c9b6a6cafe.png)
27. Click `Confirm` in the dialog


Thats it!

Gdrive will ask for your Client Id and Client Secret when using the `gdrive account add` command.



