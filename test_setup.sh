sudo -u postgres psql -d pets -c "CREATE DATABASE pets"
sudo -u postgres psql -d pets -c "DROP SCHEMA public CASCADE"
sudo -u postgres psql -d pets -c "CREATE SCHEMA public "

#load the pets database 
git clone https://github.com/Networks-Learning/stackexchange-dump-to-postgres
cd stackexchange-dump-to-postgres
pip install -r requirements.txt

python3 load_into_pg.py -s pets -d pets
#rm -r stackexchange-dump-to-postgres


#create a new user who has access to the pets database. This just allows us to hardcode a database string as everyone will use the same user.
sudo -u postgres psql -c "CREATE USER poggers"
sudo -u postgres psql -c "ALTER USER poggers WITH password 'poggers'"
sudo -u postgres psql -c "GRANT ALL ON DATABASE pets TO poggers"



#stick to singular naming convention
sudo -u postgres psql -d pets -c "ALTER TABLE badges rename to badge"      
sudo -u postgres psql -d pets -c "ALTER TABLE comments RENAME TO comment"
sudo -u postgres psql -d pets -c "ALTER TABLE postlinks RENAME TO postlink"
sudo -u postgres psql -d pets -c "ALTER TABLE posts RENAME TO post"
sudo -u postgres psql -d pets -c "ALTER TABLE tags RENAME TO tag"
sudo -u postgres psql -d pets -c "ALTER TABLE votes RENAME TO vote"

#user collides with the postgres user table
sudo -u postgres psql -d pets -c "ALTER TABLE users RENAME TO site_user"

#add foreign keys between tables
sudo -u postgres psql -d pets -c "ALTER TABLE badge ADD CONSTRAINT fk_badge_site_user FOREIGN KEY (userid) REFERENCES site_user(id) ON DELETE CASCADE;"

sudo -u postgres psql -d pets -c "ALTER TABLE comment ADD CONSTRAINT fk_comment_post FOREIGN KEY (postid) REFERENCES post(id) ON DELETE CASCADE;"
sudo -u postgres psql -d pets -c "ALTER TABLE comment ADD CONSTRAINT fk_comment_post FOREIGN KEY (postid) REFERENCES post(id) ON DELETE CASCADE;"
sudo -u postgres psql -d pets -c "ALTER TABLE comment ADD CONSTRAINT fk_comment_site_user FOREIGN KEY (userid) REFERENCES site_user(id) ON DELETE CASCADE;"

sudo -u postgres psql -d pets -c "ALTER TABLE post ADD CONSTRAINT fk_post_parent_post FOREIGN KEY (parentid) REFERENCES post(id) ON DELETE CASCADE;"
sudo -u postgres psql -d pets -c "ALTER TABLE post ADD CONSTRAINT fk_post_lasteditoruser FOREIGN KEY (lasteditoruserid) REFERENCES site_user(id) ON DELETE CASCADE;"
sudo -u postgres psql -d pets -c "ALTER TABLE post ADD CONSTRAINT fk_post_owneruser FOREIGN KEY (owneruserid) REFERENCES site_user(id) ON DELETE CASCADE;"

sudo -u postgres psql -d pets -c "ALTER TABLE posthistory ADD CONSTRAINT fk_posthistory_post FOREIGN KEY (postid) REFERENCES post(id) ON DELETE CASCADE;"

sudo -u postgres psql -d pets -c "ALTER TABLE posthistory ADD CONSTRAINT fk_posthistory_post FOREIGN KEY (postid) REFERENCES post(id) ON DELETE CASCADE;"

sudo -u postgres psql -d pets -c "ALTER TABLE postlink ADD CONSTRAINT fk_postlink_post FOREIGN KEY (posthistory) REFERENCES post(id) ON DELETE CASCADE;"
sudo -u postgres psql -d pets -c "ALTER TABLE postlink ADD CONSTRAINT fk_postlink_post FOREIGN KEY (posthistory) REFERENCES post(id) ON DELETE CASCADE;"

sudo -u postgres psql -d pets -c "ALTER TABLE tag ADD CONSTRAINT fk_tag_excerptpostid FOREIGN KEY (excerptpostid) REFERENCES post(id) ON DELETE CASCADE;"
sudo -u postgres psql -d pets -c "ALTER TABLE tag ADD CONSTRAINT fk_tag_wikipostid FOREIGN KEY (wikipostid) REFERENCES post(id) ON DELETE CASCADE;"

sudo -u postgres psql -d pets -c "ALTER TABLE vote ADD CONSTRAINT fk_vote_user FOREIGN KEY (userid) REFERENCES site_user(id) ON DELETE CASCADE;"
sudo -u postgres psql -d pets -c "ALTER TABLE vote ADD CONSTRAINT fk_vote_post FOREIGN KEY (postid) REFERENCES post(id) ON DELETE CASCADE;"

#we create some special tables to test edge cases which aren't tested by the regular database, such as a foreign key which is a primary key
sudo -u postgres psql -d pets -c "CREATE TABLE foreign_primary_key(post_id integer primary key references post(id));"
sudo -u postgres psql -d pets -c "CREATE TABLE compound_table(id1 integer, id2 integer, primary key(id1, id2));"
sudo -u postgres psql -d pets -c "INSERT INTO compound_table(id1, id2) values (1, 100);"
sudo -u postgres psql -d pets -c "CREATE TABLE compound_child_table(id integer PRIMARY KEY generated ALWAYS AS IDENTITY, parent_id1 integer, parent_id2 integer, FOREIGN KEY(parent_id1, parent_id2) REFERENCES compound_table(id1, id2));"

#delete the script used to build the database to avoid putting it in git
rm -r ./stackexchange-dump-to-postgres
