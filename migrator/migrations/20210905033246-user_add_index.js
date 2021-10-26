module.exports = {
  async up(db) {
    const User = db.collection("User");
    await User.createIndex({ email: 1 }, { name: "email" });
    await User.createIndex({ phone: 1 }, { name: "phone" });
  },

  async down(db) {
    const User = db.collection("User");
    await User.dropIndex(["email", "phone"]);
  },
};
