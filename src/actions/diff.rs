/**
 * First the user will need to provide a password and it will have to be the one they used for backup
 * We will then get a backup entry based on the source path.
 * If there is None, there is no backup entry for that path so no change happened for that path,
 * else from what we receive, we can filter out the one whose hash matches the password hash
 * If there is none, the password is incorrect.
 * 
 * If there is some, we can now traverse the target, and match files to generate the diffs.
 * 
 * How Diff will affect backup and garbage collector.
 * The diffing capability gives a version history capability.
 * So this will lead to a modification to the garbage collector functionality
 * 1. When the limit is reached, we will merge the older diff we are about to delete and merge 
 *      with the current last file before we delete.
 * 2. This means I will have to implement an algorithm that will work on this functionality. 
 *      Since we are merging the files, it shouldn't be too complicated and we can merge them.
 *      I have to figure out how the diff crate works and what I need to do to make a successful merge.
 *      Right now my problem is if the file A has some additions and file B has deletions and I just need
 *      to make sure that new file from the merge uses this logic.
 */