# Purpose
I'm moving my files from Dropbox to Google Drive. The problem
with Dropbox is that it's storing images in the cloud. Even when you are exporting your file as a markdown, image links are pointing to the cloud. Since my membership is expiring soon and I don't want to loose these files, I need to download the images included in the markdown and be able to specify where to place them.

## What will it do?
- This program is going to take as input the markdown file exported form Dropbox. As well, as the folder where the images are going to be downloaded to.
- The program is going to download the images and replace the `url` of the images to point to the resource folder where the images are..

## Command UI
```Bash
get_images --file path_to_md file --resource_folder folder_to_store_images_in/
```